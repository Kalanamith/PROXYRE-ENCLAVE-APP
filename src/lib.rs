#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

extern crate ed25519_dalek;

pub mod command_parser;
pub mod protocol_helpers;
pub mod utils;
mod proto;
use command_parser::{ClientArgs, ServerArgs};
use protocol_helpers::{recv_loop, recv_u64, send_loop, send_u64};

use nix::sys::socket::listen as listen_vsock;
use nix::sys::socket::{accept, bind, connect, shutdown, socket};
use nix::sys::socket::{AddressFamily, Shutdown, SockAddr, SockFlag, SockType};
use nix::unistd::close;
use std::convert::TryInto;
use std::os::unix::io::{AsRawFd, RawFd};
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use recrypt::api::{CryptoOps, Ed25519Ops, EncryptedValue, KeyGenOps, Plaintext, PrivateKey, PublicKey, Recrypt, TransformBlock};
use rocket::Config;
use rocket::config::{Environment, LoggingLevel};
use rocket::http::Method;


use proto::transform::{PublicKey as PPK, TransformBlock as TFB, TransformObject as TFO};
use protobuf;
use protobuf::Message;

mod models;

use rocket_contrib::json::Json;
use rocket_cors::{AllowedOrigins, Cors, CorsOptions};
use serde::{Serialize, Deserialize};
use crate::models::{EncryptedResponse, Keys, Payload, TransformedBlockResponse, TransformedObject, TransformedObjectResponse, TransformPublicKeyCollection};

extern crate rand;

const VMADDR_CID_ANY: u32 = 0xFFFFFFFF;
const BUF_MAX_LEN: usize = 32;
// Maximum number of outstanding connections in the socket's
// listen queue
const BACKLOG: usize = 128;
// Maximum number of connection attempts
const MAX_CONNECTION_ATTEMPTS: usize = 5;

struct VsockSocket {
    socket_fd: RawFd,
}

impl VsockSocket {
    fn new(socket_fd: RawFd) -> Self {
        VsockSocket { socket_fd }
    }
}

impl Drop for VsockSocket {
    fn drop(&mut self) {
        shutdown(self.socket_fd, Shutdown::Both)
            .unwrap_or_else(|e| eprintln!("Failed to shut socket down: {:?}", e));
        close(self.socket_fd).unwrap_or_else(|e| eprintln!("Failed to close socket: {:?}", e));
    }
}

impl AsRawFd for VsockSocket {
    fn as_raw_fd(&self) -> RawFd {
        self.socket_fd
    }
}

/// Initiate a connection on an AF_VSOCK socket
fn vsock_connect(cid: u32, port: u32) -> Result<VsockSocket, String> {
    let sockaddr = SockAddr::new_vsock(cid, port);
    let mut err_msg = String::new();

    for i in 0..MAX_CONNECTION_ATTEMPTS {
        let vsocket = VsockSocket::new(
            socket(
                AddressFamily::Vsock,
                SockType::Stream,
                SockFlag::empty(),
                None,
            )
            .map_err(|err| format!("Failed to create the socket: {:?}", err))?,
        );
        match connect(vsocket.as_raw_fd(), &sockaddr) {
            Ok(_) => return Ok(vsocket),
            Err(e) => err_msg = format!("Failed to connect: {}", e),
        }

        // Exponentially backoff before retrying to connect to the socket
        std::thread::sleep(std::time::Duration::from_secs(1 << i));
    }

    Err(err_msg)
}


#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
struct EncResp {
    public_key: String,
    encrypted_public_key: String,
    encrypted_private_key: String,
    user_token: String,
}

impl EncResp {
    fn new(public_key: String, encrypted_public_key: String, encrypted_private_key: String, user_token:String) -> Self {
        EncResp {
            public_key,
            encrypted_public_key,
            encrypted_private_key,
            user_token,
        }
    }
}

fn ppk_from_public_key(pubkey: &PublicKey) -> PPK {
    let mut ppk = PPK::new();
    ppk.x = Vec::from(pubkey.bytes_x_y().0.as_slice());
    ppk.y = Vec::from(pubkey.bytes_x_y().1.as_slice());
    ppk
}

fn tfb_from_params(transform_block: &TransformBlock) -> TFB {
    let transform_block_pk = ppk_from_public_key(transform_block.public_key());
    let random_transform_pk = ppk_from_public_key(transform_block.random_transform_public_key());
    let mut tbf = TFB::new();

    tbf.public_key = Some(PPK::from(transform_block_pk).into()).into();
    tbf.encrypted_temp_key = Vec::from(transform_block.encrypted_temp_key().bytes().as_slice());
    tbf.random_transform_public_key = Some(PPK::from(random_transform_pk.clone()).into()).into();
    tbf.encrypted_random_transform_temp_key = Vec::from(
        transform_block
            .encrypted_random_transform_temp_key()
            .bytes()
            .as_slice(),
    );
    tbf
}

fn trans_response_from_params(
    ephemeral_pk: &PublicKey,
    transform_block: &TransformBlock,
    transblock: &TFB,
    transformed_obj: &TFO,
) -> TransformedObject {
    let random_transform_pk = ppk_from_public_key(transform_block.random_transform_public_key());

    TransformedObject {
        // TODO: Needs to construct additional Struct to split and show values
        ephemeral_public_key: TransformPublicKeyCollection {
            public_key_x: hex::encode(Vec::from(ephemeral_pk.bytes_x_y().0.as_slice())),
            public_key_y: hex::encode(Vec::from(ephemeral_pk.bytes_x_y().1.as_slice())),
        },
        encrypted_message: hex::encode(&transformed_obj.encrypted_message),
        auth_hash: hex::encode(&transformed_obj.auth_hash),
        transform_blocks: TransformedBlockResponse {
            public_key: TransformPublicKeyCollection {
                public_key_x: hex::encode(&transblock.public_key.x),
                public_key_y: hex::encode(&transblock.public_key.y),
            },
            encrypted_temp_key: hex::encode(&transblock.encrypted_temp_key),
            encrypted_random_transform_temp_key: hex::encode(
                &transblock.encrypted_random_transform_temp_key,
            ),
            random_transform_public_key: TransformPublicKeyCollection {
                public_key_x: hex::encode(random_transform_pk.x),
                public_key_y: hex::encode(random_transform_pk.y),
            },
        },
        public_signing_key: hex::encode(&transformed_obj.public_signing_key),
        ed25519_signature: hex::encode(&transformed_obj.ed25519_signature),
    }
}

fn hardcoded_plaintext() -> Plaintext {
    // Harcoded Plaintext generated with recrypt.gen_plaintext()
    let msg = vec![
        49, 99, 205, 79, 20, 51, 152, 222, 138, 58, 111, 88, 32, 103, 216, 127, 141, 68, 119, 226,
        101, 44, 246, 48, 34, 10, 5, 106, 222, 237, 4, 240, 70, 22, 249, 34, 140, 196, 6, 50, 48,
        137, 242, 131, 79, 86, 98, 253, 111, 197, 207, 57, 214, 237, 155, 43, 242, 162, 215, 54,
        71, 78, 21, 188, 85, 199, 119, 122, 60, 123, 106, 118, 59, 201, 14, 97, 123, 136, 114, 136,
        146, 129, 43, 44, 53, 87, 155, 102, 84, 223, 126, 100, 58, 113, 117, 210, 108, 14, 5, 221,
        113, 205, 103, 142, 231, 196, 0, 5, 193, 107, 205, 106, 135, 23, 159, 204, 46, 93, 10, 205,
        146, 223, 188, 186, 86, 102, 127, 54, 63, 6, 77, 198, 119, 248, 37, 80, 133, 173, 223, 125,
        229, 121, 26, 228, 198, 77, 73, 252, 104, 71, 84, 149, 74, 205, 70, 39, 120, 145, 127, 222,
        143, 179, 94, 27, 56, 17, 246, 167, 132, 163, 213, 253, 178, 109, 204, 140, 11, 187, 17,
        93, 61, 229, 44, 111, 221, 179, 189, 247, 212, 142, 148, 150, 56, 89, 97, 57, 25, 250, 172,
        72, 79, 154, 84, 177, 36, 22, 54, 184, 101, 49, 122, 139, 178, 173, 147, 131, 154, 14, 3,
        65, 26, 241, 216, 132, 48, 55, 240, 36, 5, 250, 120, 199, 161, 73, 87, 212, 119, 163, 101,
        142, 223, 142, 208, 235, 47, 183, 105, 84, 143, 150, 58, 233, 70, 32, 174, 19, 98, 214, 40,
        73, 132, 28, 129, 200, 14, 224, 42, 183, 47, 147, 7, 132, 200, 180, 121, 215, 109, 7, 169,
        143, 103, 182, 155, 129, 185, 203, 28, 154, 100, 232, 163, 201, 52, 58, 173, 37, 33, 197,
        111, 162, 144, 154, 79, 9, 33, 196, 166, 39, 4, 173, 102, 90, 134, 145, 42, 221, 48, 165,
        92, 148, 36, 247, 160, 73, 197, 53, 7, 187, 49, 138, 40, 146, 176, 70, 77, 220, 23, 55, 88,
        155, 47, 71, 110, 61, 133, 68, 239, 123, 36, 150, 19, 3, 23, 208, 95, 123, 245, 11, 7, 5,
        162, 210, 132, 129, 160, 209, 40, 231, 90, 62, 25, 24, 140, 43, 253, 112, 131, 168, 133,
        232, 26, 32, 36, 49,
    ];

    Plaintext::new_from_slice(&msg).unwrap()
}

#[get("/")]
fn get_root() -> Json<String> {
    Json(String::from("Hola!!!"))
}


#[post("/", format = "json", data = "<payload>")]
fn upload_content(payload: Json<Payload>) -> Json<String> {
    // TODO: figure this out
    println!("payload --- {:?}", payload);
    println!();

    Json(String::from("upload_content - work in progress"))
}

#[post("/", format = "json", data = "<payload>")]
fn fetch_content(payload: Json<Payload>) -> Json<TransformedObjectResponse> {
    println!("payload --- {:?}", payload);
    println!();

    // Content Creator's Private Key
    let initial_private_key = PrivateKey::new_from_slice(&payload.initial_private_key).unwrap();

    // Content Creator's Public Key
    let owner_public_key =
        PublicKey::new_from_slice((&payload.initial_public_key_x, &payload.initial_public_key_y))
            .unwrap();

    // Bob's PK
    let delegatee_public_key = PublicKey::new_from_slice((
        &payload.delegatee_public_key_x,
        &payload.delegatee_public_key_y,
    ))
        .unwrap();

    // *********************************************************************
    let recrypt = Recrypt::new();
    let signing_keypair = recrypt.generate_ed25519_key_pair();
    // let plain_text = recrypt.gen_plaintext();

    let plain_text = hardcoded_plaintext();
    let mut display = TransformedObject::default();

    let encrypted_val = recrypt
        .encrypt(
            &plain_text,
            &owner_public_key, // initial public key
            &signing_keypair,  // signer key pair
        )
        .unwrap();

    // for this we need Bos,s public
    let initial_to_target_transform_key = recrypt
        .generate_transform_key(
            &initial_private_key,  // initial private key
            &delegatee_public_key, // target public key
            &signing_keypair,
        )
        .unwrap();

    // Transform the plaintext to be encrypted to the target!
    // The data is _not_ decrypted here. Simply transformed!
    let transformed_val = recrypt
        .transform(
            encrypted_val,
            initial_to_target_transform_key,
            &signing_keypair,
        )
        .unwrap();

    let mut to = TFO::new();

    println!("transformed_val {:?}", transformed_val);
    println!();

    if let EncryptedValue::TransformedValue {
        ephemeral_public_key: ep,
        encrypted_message: em,
        auth_hash: ah,
        transform_blocks: tb,
        public_signing_key: ps,
        signature: sg,
    } = transformed_val
    {
        let ppk = ppk_from_public_key(&ep);
        let transblock = tfb_from_params(tb.first());

        // End assigning

        to.ephemeral_public_key = Some(PPK::from(ppk).into()).into();
        to.encrypted_message = Vec::from(em.bytes().as_slice());
        to.auth_hash = Vec::from(ah.bytes().as_slice());
        to.transform_blocks = Some(TFB::from(transblock.clone()).into()).into();
        to.public_signing_key = Vec::from(ps.bytes().as_slice());
        to.ed25519_signature = Vec::from(sg.bytes().as_slice());

        println!("*************************************************************");
        println!("0TGFBLOKC:- {:?}", to.transform_blocks);
        println!("*************************************************************");

        // TODO: We might need this structure to deserialize and reconstruct the transform object
        display = trans_response_from_params(&ep, tb.first(), &transblock, &to);

        println!("TransformedObject as Hex values \n {:?}", display);
    };

    println!("Transform Object");

    // *********************************************************************************************

    let tfo_bytes = to.write_to_bytes().unwrap();

    let response = EncryptedResponse {
        sender_public_key: hex::encode(&payload.initial_public_key_x),
        encrypted_resource: hex::encode(&payload.resource),
        transformed: hex::encode(&tfo_bytes),
        // Passing this for test purposes
        transformed_response: display,
    };

    let tr = TransformedObjectResponse {
        transformed_object: hex::encode(&tfo_bytes),
    };

    Json(TransformedObjectResponse {
        transformed_object: hex::encode(&tfo_bytes),
    })
}
/// Gets Keys
#[get("/")]
fn get_key_pair() -> Json<Keys> {
    let recrypt = Recrypt::new();
    let (private_key, public_key) = recrypt.generate_key_pair().unwrap();

    println!("Public Key {:?}", public_key);
    println!();

    let pk = PPK::new();
    let bbs = protobuf::Message::write_to_bytes(&pk).unwrap();

    println!("Public Key ---- {:?}", bbs);
    println!();

    let keys = Keys {
        private_key: Vec::from(private_key.bytes().as_slice()),
        public_key_x: Vec::from(public_key.bytes_x_y().0.as_slice()),
        public_key_y: Vec::from(public_key.bytes_x_y().1.as_slice()),
    };
    Json(keys)
}
/// Starting point of the Enclave Parent Instance
pub fn client(_args: ClientArgs) -> Result<(), String> {
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Patch]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true);

    let config = Config::build(Environment::Staging)
        .address("0.0.0.0")
        .port(8000)
        .workers(4)
        .log_level(LoggingLevel::Debug)
        .keep_alive(5)
        .read_timeout(5)
        .write_timeout(5)
        .unwrap();

    rocket::ignite().attach(cors.to_cors().unwrap())
        .mount("/", routes![get_root])
        .mount("/get-keys", routes![get_key_pair]) // get
        .mount("/upload-content", routes![upload_content]) // post
        .mount("/fetch-content", routes![fetch_content]) // post
        .launch();

    Ok(())
}

/// Accept connections on a certain port and print
/// the received data
pub fn server(args: ServerArgs) -> Result<(), String> {
    let socket_fd = socket(
        AddressFamily::Vsock,
        SockType::Stream,
        SockFlag::empty(),
        None,
    )
    .map_err(|err| format!("Create socket failed: {:?}", err))?;

    let sockaddr = SockAddr::new_vsock(VMADDR_CID_ANY, args.port);

    bind(socket_fd, &sockaddr).map_err(|err| format!("Bind failed: {:?}", err))?;

    listen_vsock(socket_fd, BACKLOG).map_err(|err| format!("Listen failed: {:?}", err))?;

    loop {

        // Read Key Generation Request

        // Read Encryption Request





        let fd = accept(socket_fd).map_err(|err| format!("Accept failed: {:?}", err))?;

        let len = recv_u64(fd)?;
        let mut buf = [0u8; BUF_MAX_LEN];
        recv_loop(fd, &mut buf, len)?;

        let mut csprng = OsRng{};
        let keypair: Keypair = Keypair::generate(&mut csprng);

        let ed_public_key = keypair.public.as_bytes();
        let ed_private_key = keypair.secret.as_bytes();

        let received_public_key =  ecies_ed25519::PublicKey::from_bytes(&buf.as_slice()).unwrap();

        let encrypted_1 = ecies_ed25519::encrypt(&received_public_key, ed_public_key, &mut csprng).unwrap();
        let encrypted_2 = ecies_ed25519::encrypt(&received_public_key, ed_private_key, &mut csprng).unwrap();

        println!("Received clients public key in bytes  {:?}", buf.clone());
        println!("Clients Public Key  {:?}", hex::encode(&received_public_key));

        println!("ED25519 Generated Public Key {:?}", hex::encode(&ed_public_key));
        println!("ED25519 Encrypted private key key with Clients Public Key {:?} ", hex::encode(&encrypted_2));
        println!("ED25519 Encrypted public key with Clients Public Key  {:?}", hex::encode(&encrypted_1));
    }
}
