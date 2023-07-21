use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Deserialize, Clone)]
pub struct Payload {
    pub initial_private_key: Vec<u8>,
    pub initial_public_key_x: Vec<u8>,
    pub initial_public_key_y: Vec<u8>,
    pub delegatee_public_key_x: Vec<u8>,
    pub delegatee_public_key_y: Vec<u8>,
    pub resource: Vec<u8>,
}

// Only for logs
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Default)]
pub struct TransformedBlockResponse {
    pub public_key: TransformPublicKeyCollection,
    pub encrypted_temp_key: String,
    pub encrypted_random_transform_temp_key: String,
    pub random_transform_public_key: TransformPublicKeyCollection,
}

// Only for logs
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Default)]
pub struct TransformedObject {
    pub ephemeral_public_key: TransformPublicKeyCollection,
    pub encrypted_message: String,
    pub auth_hash: String,
    pub transform_blocks: TransformedBlockResponse,
    pub public_signing_key: String,
    pub ed25519_signature: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Default)]
pub struct TransformPublicKeyCollection {
    pub public_key_x: String,
    pub public_key_y: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct EncryptedResponse {
    pub sender_public_key: String,
    pub encrypted_resource: String,
    pub transformed: String,
    pub transformed_response: TransformedObject,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Default)]
pub struct Keys {
    pub private_key: Vec<u8>,
    pub public_key_x: Vec<u8>,
    pub public_key_y: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Default)]
pub struct TransformedObjectResponse {
    pub transformed_object: String,
}