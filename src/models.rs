use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;

    // Test Payload struct
    #[test]
    fn test_payload_creation() {
        let payload = Payload {
            initial_private_key: vec![1, 2, 3],
            initial_public_key_x: vec![4, 5, 6],
            initial_public_key_y: vec![7, 8, 9],
            delegatee_public_key_x: vec![10, 11, 12],
            delegatee_public_key_y: vec![13, 14, 15],
            resource: vec![16, 17, 18],
        };

        assert_eq!(payload.initial_private_key, vec![1, 2, 3]);
        assert_eq!(payload.resource, vec![16, 17, 18]);
    }

    #[test]
    fn test_payload_debug() {
        let payload = Payload {
            initial_private_key: vec![1],
            initial_public_key_x: vec![2],
            initial_public_key_y: vec![3],
            delegatee_public_key_x: vec![4],
            delegatee_public_key_y: vec![5],
            resource: vec![6],
        };

        let debug_str = format!("{payload:?}");
        assert!(debug_str.contains("Payload"));
        assert!(debug_str.contains("initial_private_key"));
        assert!(debug_str.contains("resource"));
    }

    // Test Keys struct
    #[test]
    fn test_keys_creation() {
        let keys = Keys {
            private_key: vec![1, 2, 3, 4],
            public_key_x: vec![5, 6, 7, 8],
            public_key_y: vec![9, 10, 11, 12],
        };

        assert_eq!(keys.private_key.len(), 4);
        assert_eq!(keys.public_key_x.len(), 4);
        assert_eq!(keys.public_key_y.len(), 4);
    }

    #[test]
    fn test_keys_default() {
        let keys = Keys::default();
        assert!(keys.private_key.is_empty());
        assert!(keys.public_key_x.is_empty());
        assert!(keys.public_key_y.is_empty());
    }

    // Test TransformedObjectResponse struct
    #[test]
    fn test_transformed_object_response_creation() {
        let response = TransformedObjectResponse {
            transformed_object: "test_data".to_string(),
        };

        assert_eq!(response.transformed_object, "test_data");
    }

    #[test]
    fn test_transformed_object_response_default() {
        let response = TransformedObjectResponse::default();
        assert!(response.transformed_object.is_empty());
    }

    // Test TransformPublicKeyCollection struct
    #[test]
    fn test_transform_public_key_collection_creation() {
        let collection = TransformPublicKeyCollection {
            public_key_x: "12345".to_string(),
            public_key_y: "67890".to_string(),
        };

        assert_eq!(collection.public_key_x, "12345");
        assert_eq!(collection.public_key_y, "67890");
    }

    #[test]
    fn test_transform_public_key_collection_default() {
        let collection = TransformPublicKeyCollection::default();
        assert!(collection.public_key_x.is_empty());
        assert!(collection.public_key_y.is_empty());
    }

    // Test TransformedObject struct
    #[test]
    fn test_transformed_object_creation() {
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

    #[test]
    fn test_transformed_object_default() {
        let transformed = TransformedObject::default();
        assert!(transformed.ephemeral_public_key.public_key_x.is_empty());
        assert!(transformed.encrypted_message.is_empty());
        assert!(transformed.auth_hash.is_empty());
        assert!(transformed.public_signing_key.is_empty());
        assert!(transformed.ed25519_signature.is_empty());
    }

    // Test TransformedBlockResponse struct
    #[test]
    fn test_transformed_block_response_creation() {
        let block_response = TransformedBlockResponse {
            public_key: TransformPublicKeyCollection {
                public_key_x: "pk_x".to_string(),
                public_key_y: "pk_y".to_string(),
            },
            encrypted_temp_key: "temp_key".to_string(),
            encrypted_random_transform_temp_key: "random_temp_key".to_string(),
            random_transform_public_key: TransformPublicKeyCollection {
                public_key_x: "rt_pk_x".to_string(),
                public_key_y: "rt_pk_y".to_string(),
            },
        };

        assert_eq!(block_response.encrypted_temp_key, "temp_key");
        assert_eq!(block_response.public_key.public_key_x, "pk_x");
        assert_eq!(
            block_response.random_transform_public_key.public_key_x,
            "rt_pk_x"
        );
    }

    #[test]
    fn test_transformed_block_response_default() {
        let block_response = TransformedBlockResponse::default();
        assert!(block_response.encrypted_temp_key.is_empty());
        assert!(block_response
            .encrypted_random_transform_temp_key
            .is_empty());
        assert!(block_response.public_key.public_key_x.is_empty());
        assert!(block_response
            .random_transform_public_key
            .public_key_x
            .is_empty());
    }

    // Test EncryptedResponse struct
    #[test]
    fn test_encrypted_response_creation() {
        let encrypted_response = EncryptedResponse {
            sender_public_key: "sender_pk".to_string(),
            encrypted_resource: "encrypted_res".to_string(),
            transformed: "transformed_data".to_string(),
            transformed_response: TransformedObject::default(),
        };

        assert_eq!(encrypted_response.sender_public_key, "sender_pk");
        assert_eq!(encrypted_response.encrypted_resource, "encrypted_res");
        assert_eq!(encrypted_response.transformed, "transformed_data");
    }

    // Test equality implementations
    #[test]
    fn test_payload_equality() {
        let payload1 = Payload {
            initial_private_key: vec![1, 2, 3],
            initial_public_key_x: vec![4, 5, 6],
            initial_public_key_y: vec![7, 8, 9],
            delegatee_public_key_x: vec![10, 11, 12],
            delegatee_public_key_y: vec![13, 14, 15],
            resource: vec![16, 17, 18],
        };

        let payload2 = payload1.clone();
        let payload3 = Payload {
            initial_private_key: vec![99],
            initial_public_key_x: vec![4, 5, 6],
            initial_public_key_y: vec![7, 8, 9],
            delegatee_public_key_x: vec![10, 11, 12],
            delegatee_public_key_y: vec![13, 14, 15],
            resource: vec![16, 17, 18],
        };

        assert_eq!(payload1, payload2);
        assert_ne!(payload1, payload3);
    }

    #[test]
    fn test_keys_equality() {
        let keys1 = Keys {
            private_key: vec![1, 2, 3],
            public_key_x: vec![4, 5, 6],
            public_key_y: vec![7, 8, 9],
        };

        let keys2 = keys1.clone();
        let keys3 = Keys {
            private_key: vec![99, 2, 3],
            public_key_x: vec![4, 5, 6],
            public_key_y: vec![7, 8, 9],
        };

        assert_eq!(keys1, keys2);
        assert_ne!(keys1, keys3);
    }

    // Test Clone implementations
    #[test]
    fn test_payload_clone() {
        let payload1 = Payload {
            initial_private_key: vec![1, 2, 3],
            initial_public_key_x: vec![4, 5, 6],
            initial_public_key_y: vec![7, 8, 9],
            delegatee_public_key_x: vec![10, 11, 12],
            delegatee_public_key_y: vec![13, 14, 15],
            resource: vec![16, 17, 18],
        };

        let payload2 = payload1.clone();
        assert_eq!(payload1, payload2);
        assert_eq!(payload1.initial_private_key, payload2.initial_private_key);
    }

    #[test]
    fn test_keys_clone() {
        let keys1 = Keys {
            private_key: vec![1, 2, 3],
            public_key_x: vec![4, 5, 6],
            public_key_y: vec![7, 8, 9],
        };

        let keys2 = keys1.clone();
        assert_eq!(keys1, keys2);
        assert_eq!(keys1.private_key, keys2.private_key);
    }
}
