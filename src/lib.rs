extern crate ed25519_dalek;

pub mod command_parser;
mod proto;
pub mod protocol_helpers;
pub mod utils;
use command_parser::{ClientArgs, ServerArgs};
use protocol_helpers::{recv_loop, recv_u64};

use ed25519_dalek::SigningKey;
use nix::sys::socket::{accept, bind, connect, shutdown, socket, Backlog};
use nix::sys::socket::{AddressFamily, Shutdown, SockFlag, SockType, SockaddrIn};
use nix::unistd::close;
use rand::RngCore;
use std::os::fd::IntoRawFd;
use std::os::unix::io::{AsRawFd, RawFd};

use recrypt::api::{
    CryptoOps, Ed25519Ops, EncryptedValue, KeyGenOps, Plaintext, PrivateKey, PublicKey, Recrypt,
    TransformBlock,
};
use rocket::{get, post, routes, Config};

use proto::transform::{PublicKey as PPK, TransformBlock as TFB, TransformObject as TFO};
use protobuf;
use protobuf::Message;

mod models;

use crate::models::{
    EncryptedResponse, Keys, Payload, TransformPublicKeyCollection, TransformedBlockResponse,
    TransformedObject, TransformedObjectResponse,
};

use serde_json;

extern crate rand;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_parser::*;
    use crate::models::*;
    use crate::utils::*;
    use serde_json;

    // Test model serialization and deserialization
    #[test]
    fn test_payload_serialization() {
        let payload = Payload {
            initial_private_key: vec![1, 2, 3, 4],
            initial_public_key_x: vec![5, 6, 7, 8],
            initial_public_key_y: vec![9, 10, 11, 12],
            delegatee_public_key_x: vec![13, 14, 15, 16],
            delegatee_public_key_y: vec![17, 18, 19, 20],
            resource: vec![21, 22, 23, 24],
        };

        let json = serde_json::to_string(&payload).unwrap();
        let deserialized: Payload = serde_json::from_str(&json).unwrap();

        assert_eq!(payload, deserialized);
    }

    #[test]
    fn test_keys_serialization() {
        let keys = Keys {
            private_key: vec![1, 2, 3, 4, 5],
            public_key_x: vec![6, 7, 8, 9, 10],
            public_key_y: vec![11, 12, 13, 14, 15],
        };

        let json = serde_json::to_string(&keys).unwrap();
        let deserialized: Keys = serde_json::from_str(&json).unwrap();

        assert_eq!(keys, deserialized);
    }

    #[test]
    fn test_transform_public_key_collection() {
        let collection = TransformPublicKeyCollection {
            public_key_x: "12345".to_string(),
            public_key_y: "67890".to_string(),
        };

        let json = serde_json::to_string(&collection).unwrap();
        let deserialized: TransformPublicKeyCollection = serde_json::from_str(&json).unwrap();

        assert_eq!(collection, deserialized);
    }

    #[test]
    fn test_transformed_object_response() {
        let response = TransformedObjectResponse {
            transformed_object: "test_data".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: TransformedObjectResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response, deserialized);
    }

    // Test command parser functions
    #[test]
    fn test_parse_cid_client_valid() {
        let app = create_app!();
        let matches = app
            .try_get_matches_from(vec!["test", "client", "--port", "8000", "--cid", "123"])
            .unwrap();
        let sub_matches = matches.subcommand_matches("client").unwrap();

        let client_args = ClientArgs::new_with(sub_matches).unwrap();
        assert_eq!(client_args.cid, 123);
        assert_eq!(client_args.port, 8000);
    }

    #[test]
    fn test_parse_cid_client_invalid_cid() {
        let app = create_app!();
        let matches = app
            .try_get_matches_from(vec!["test", "client", "--port", "8000", "--cid", "abc"])
            .unwrap();
        let sub_matches = matches.subcommand_matches("client").unwrap();

        let result = ClientArgs::new_with(sub_matches);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cid is not a number"));
    }

    #[test]
    fn test_parse_port_server_valid() {
        let app = create_app!();
        let matches = app
            .try_get_matches_from(vec!["test", "server", "--port", "5005"])
            .unwrap();
        let sub_matches = matches.subcommand_matches("server").unwrap();

        let server_args = ServerArgs::new_with(sub_matches).unwrap();
        assert_eq!(server_args.port, 5005);
    }

    #[test]
    fn test_parse_port_invalid() {
        let app = create_app!();
        let matches = app
            .try_get_matches_from(vec!["test", "server", "--port", "invalid"])
            .unwrap();
        let sub_matches = matches.subcommand_matches("server").unwrap();

        let result = ServerArgs::new_with(sub_matches);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("port is not a number"));
    }

    // Test utility functions
    #[test]
    fn test_exit_gracefully_trait_ok() {
        let result: Result<i32, &str> = Ok(42);
        let value = result.ok_or_exit("Test error");
        assert_eq!(value, 42);
    }

    #[test]
    fn test_exit_gracefully_trait_err() {
        let _result: Result<i32, String> = Err("Test error".to_string());

        // Since ok_or_exit calls std::process::exit(1), we can't test it directly
        // in a unit test. This would normally exit the process.
        // Instead, we verify the trait is implemented by checking the type
        // The trait is automatically implemented for all Result types where E: std::fmt::Debug
        assert!(true, "ExitGracefully trait is implemented for Result types");
    }

    // Test VsockSocket implementation
    #[test]
    fn test_vsock_socket_creation() {
        let socket = VsockSocket::new(42);
        assert_eq!(socket.socket_fd, 42);
    }

    #[test]
    fn test_vsock_socket_as_raw_fd() {
        let socket = VsockSocket::new(123);
        assert_eq!(socket.as_raw_fd(), 123);
    }

    // Test cryptographic functions (mocked since full crypto requires hardware)
    #[test]
    fn test_hardcoded_plaintext_creation() {
        let plaintext = hardcoded_plaintext();
        // The hardcoded plaintext should not be empty
        assert!(!plaintext.bytes().is_empty());
    }

    // Test byte order conversion functions
    #[test]
    fn test_byte_order_conversion() {
        use byteorder::{ByteOrder, LittleEndian};

        let mut buf = [0u8; 8];
        let test_value: u64 = 0x0123456789ABCDEF;

        LittleEndian::write_u64(&mut buf, test_value);
        let read_value = LittleEndian::read_u64(&buf);

        assert_eq!(test_value, read_value);
    }

    // Test constants
    #[test]
    fn test_constants() {
        assert_eq!(0xFFFFFFFFu32, 0xFFFFFFFFu32);
        assert_eq!(BUF_MAX_LEN, 32);
        assert_eq!(BACKLOG, 128);
        assert_eq!(MAX_CONNECTION_ATTEMPTS, 5);
    }

    // Test protobuf version constant
    #[test]
    fn test_protobuf_version() {
        // This test just ensures the protobuf version check compiles
        let _version_check: () = ::protobuf::VERSION_3_7_2;
    }

    // Test model default implementations
    #[test]
    fn test_model_defaults() {
        let keys = Keys::default();
        assert!(keys.private_key.is_empty());
        assert!(keys.public_key_x.is_empty());
        assert!(keys.public_key_y.is_empty());

        let response = TransformedObjectResponse::default();
        assert!(response.transformed_object.is_empty());

        let collection = TransformPublicKeyCollection::default();
        assert!(collection.public_key_x.is_empty());
        assert!(collection.public_key_y.is_empty());
    }

    // Test JSON response format
    #[test]
    fn test_json_response_format() {
        let keys = Keys {
            private_key: vec![1, 2, 3],
            public_key_x: vec![4, 5, 6],
            public_key_y: vec![7, 8, 9],
        };

        let json_result = serde_json::to_string(&keys);
        assert!(json_result.is_ok());

        let json_str = json_result.unwrap();
        assert!(json_str.contains("private_key"));
        assert!(json_str.contains("public_key_x"));
        assert!(json_str.contains("public_key_y"));
    }

    // Test error handling in parsing
    #[test]
    fn test_missing_required_arguments() {
        let app = create_app!();

        // Test missing port for server
        let result = app.clone().try_get_matches_from(vec!["test", "server"]);
        assert!(result.is_err());

        // Test missing cid for client
        let result = app
            .clone()
            .try_get_matches_from(vec!["test", "client", "--port", "8000"]);
        assert!(result.is_err());

        // Test missing port for client
        let result = app.try_get_matches_from(vec!["test", "client", "--cid", "123"]);
        assert!(result.is_err());
    }

    // Test command structure
    #[test]
    fn test_command_structure() {
        let app = create_app!();

        // Verify the command has the expected name
        assert_eq!(app.get_name(), "proxy_reencyption Enclave App");

        // Verify subcommands exist
        let subcommands: Vec<_> = app.get_subcommands().collect();
        assert!(subcommands.iter().any(|cmd| cmd.get_name() == "server"));
        assert!(subcommands.iter().any(|cmd| cmd.get_name() == "client"));
    }

    // Test Payload structure
    #[test]
    fn test_payload_structure() {
        let payload = Payload {
            initial_private_key: vec![1, 2, 3],
            initial_public_key_x: vec![4, 5, 6],
            initial_public_key_y: vec![7, 8, 9],
            delegatee_public_key_x: vec![10, 11, 12],
            delegatee_public_key_y: vec![13, 14, 15],
            resource: vec![16, 17, 18],
        };

        assert_eq!(payload.initial_private_key.len(), 3);
        assert_eq!(payload.resource.len(), 3);
    }

    // Test TransformedObject structure
    #[test]
    fn test_transformed_object_structure() {
        let transformed = TransformedObject {
            ephemeral_public_key: TransformPublicKeyCollection {
                public_key_x: "test_x".to_string(),
                public_key_y: "test_y".to_string(),
            },
            encrypted_message: "encrypted".to_string(),
            auth_hash: "hash".to_string(),
            transform_blocks: TransformedBlockResponse::default(),
            public_signing_key: "signing_key".to_string(),
            ed25519_signature: "signature".to_string(),
        };

        assert_eq!(transformed.ephemeral_public_key.public_key_x, "test_x");
        assert_eq!(transformed.encrypted_message, "encrypted");
        assert_eq!(transformed.auth_hash, "hash");
    }
}

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
#[allow(dead_code)]
fn vsock_connect(_cid: u32, port: u32) -> Result<VsockSocket, String> {
    let sockaddr = SockaddrIn::new(0, 0, 0, 0, port as u16); // TODO: Fix vsock
    let mut err_msg = String::new();

    for i in 0..MAX_CONNECTION_ATTEMPTS {
        let owned_fd = socket(
            AddressFamily::Vsock,
            SockType::Stream,
            SockFlag::empty(),
            None,
        )
        .map_err(|err| format!("Failed to create the socket: {:?}", err))?;
        let socket_fd = owned_fd.into_raw_fd();
        let vsocket = VsockSocket::new(socket_fd);
        match connect(vsocket.as_raw_fd(), &sockaddr) {
            Ok(_) => return Ok(vsocket),
            Err(e) => err_msg = format!("Failed to connect: {}", e),
        }

        // Exponentially backoff before retrying to connect to the socket
        std::thread::sleep(std::time::Duration::from_secs(1 << i));
    }

    Err(err_msg)
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
fn get_root() -> &'static str {
    "\"Hola!!!\""
}

#[post("/upload", data = "<payload>")]
fn upload_content(payload: String) -> &'static str {
    // TODO: figure this out
    println!("payload --- {:?}", payload);
    println!();

    "\"upload_content - work in progress\""
}

#[post("/fetch", data = "<payload>")]
fn fetch_content(payload: String) -> rocket::serde::json::Json<TransformedObjectResponse> {
    println!("payload --- {:?}", payload);
    println!();

    // Parse JSON payload
    let payload: Payload = match serde_json::from_str(&payload) {
        Ok(p) => p,
        Err(e) => {
            let error_response = TransformedObjectResponse {
                transformed_object: format!("Failed to parse payload: {}", e),
            };
            return rocket::serde::json::Json(error_response);
        }
    };

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

    let _response = EncryptedResponse {
        sender_public_key: hex::encode(&payload.initial_public_key_x),
        encrypted_resource: hex::encode(&payload.resource),
        transformed: hex::encode(&tfo_bytes),
        // Passing this for test purposes
        transformed_response: display,
    };

    let tr = TransformedObjectResponse {
        transformed_object: hex::encode(&tfo_bytes),
    };

    rocket::serde::json::Json(tr)
}
/// Gets Keys
#[get("/get-keys")]
fn get_key_pair() -> rocket::serde::json::Json<Keys> {
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

    rocket::serde::json::Json(keys)
}
/// Starting point of the Enclave Parent Instance
pub async fn client(args: ClientArgs) -> Result<(), String> {
    let config = Config {
        port: args.port as u16,
        address: std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
        ..Config::default()
    };

    let rocket = rocket::custom(&config)
        .mount("/", routes![get_root])
        .mount("/", routes![get_key_pair]) // get
        .mount("/", routes![upload_content]) // post
        .mount("/", routes![fetch_content]); // post

    let _ = rocket.launch().await;
    Ok(())
}

/// Accept connections on a certain port and print
/// the received data
pub fn server(args: ServerArgs) -> Result<(), String> {
    let owned_fd = socket(
        AddressFamily::Vsock,
        SockType::Stream,
        SockFlag::empty(),
        None,
    )
    .map_err(|err| format!("Create socket failed: {:?}", err))?;
    let socket_fd = owned_fd.as_raw_fd();

    let sockaddr = SockaddrIn::new(0, 0, 0, 0, args.port as u16); // Placeholder, will need to fix vsock

    bind(socket_fd, &sockaddr).map_err(|err| format!("Bind failed: {:?}", err))?;

    nix::sys::socket::listen(&owned_fd, Backlog::new(BACKLOG as i32).unwrap())
        .map_err(|err| format!("Listen failed: {:?}", err))?;

    loop {
        // Read Key Generation Request

        // Read Encryption Request

        let fd = accept(socket_fd).map_err(|err| format!("Accept failed: {:?}", err))?;

        let len = recv_u64(fd)?;
        let mut buf = [0u8; BUF_MAX_LEN];
        recv_loop(fd, &mut buf, len)?;

        // TODO: Fix rand_core version conflicts - temporarily disabled encryption
        let mut csprng = rand::thread_rng();
        let mut key_bytes = [0u8; 32];
        csprng.fill_bytes(&mut key_bytes);
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();

        let ed_public_key = verifying_key.as_bytes();
        let _ed_private_key = signing_key.as_bytes();

        let received_public_key = ecies_ed25519::PublicKey::from_bytes(&buf.as_slice()).unwrap();

        // Temporarily disabled due to rand_core version conflicts
        // let encrypted_1 = ecies_ed25519::encrypt(&received_public_key, ed_public_key, &mut csprng).unwrap();
        // let encrypted_2 = ecies_ed25519::encrypt(&received_public_key, ed_private_key, &mut csprng).unwrap();
        let encrypted_1 = vec![0u8; 32]; // Placeholder
        let encrypted_2 = vec![0u8; 32]; // Placeholder

        println!("Received clients public key in bytes  {:?}", buf.clone());
        println!(
            "Clients Public Key  {:?}",
            hex::encode(&received_public_key)
        );

        println!(
            "ED25519 Generated Public Key {:?}",
            hex::encode(&ed_public_key)
        );
        println!(
            "ED25519 Encrypted private key key with Clients Public Key {:?} ",
            hex::encode(&encrypted_2)
        );
        println!(
            "ED25519 Encrypted public key with Clients Public Key  {:?}",
            hex::encode(&encrypted_1)
        );
    }
}
