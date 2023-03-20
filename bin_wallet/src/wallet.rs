// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

// This file implements the Wallet struct and related methods.
// The wallet has one key task: to sign a message using the private key.
// The wallet also has a method to verify the signature for debugging purposes. Verification does not involve the private key.
// The actual verification of the signature should be implemented in the lib_chain module.
// You can see detailed instructions in the comments below.
// You can also look at the unit tests in ./main.rs to understand the expected behavior of the wallet.

use rsa::{RsaPublicKey, RsaPrivateKey};
use rsa::pkcs1::{EncodeRsaPublicKey, EncodeRsaPrivateKey, DecodeRsaPublicKey, DecodeRsaPrivateKey};
use rsa::pkcs1v15::{SigningKey, VerifyingKey};
use rsa::signature::{RandomizedSigner, Signature, Verifier};

use serde::{Serialize, Deserialize};
use sha2::{Sha256};
use base64ct::{Base64, Encoding};

/// A wallet that stores the key pairs. Most importantly, the private key.
/// For the format of the key, you can check the unit test at ./main.rs:test_bin_wallet_signing_and_verifying
/// to see how the key is loaded and used.
#[derive(Serialize, Deserialize)]
pub struct Wallet {
    /// Friendly name of the user. Doesn't matter what it is.
    pub user_name: String,
    /// The private key in PEM format
    pub priv_key_pem: String,
    /// The public key in PEM format
    pub pub_key_pem: String
}


impl Wallet {

    /// Default function
    pub fn default() -> Wallet {
        Wallet{user_name: String::from(""),priv_key_pem: String::from(""),pub_key_pem: String::from("")}
    }


    /// Create a new wallet with a given user name and key size.
    /// It will generate a new pair of keys.
    /// During the evaluation, you don't need to generate new keys.
    pub fn new(user_name: String, bits: usize) -> Wallet {
        // Generate new key pairs, and return as a wallet
        let mut rng = rand::thread_rng();
        let new_private_key = RsaPrivateKey::new(&mut rng, bits).expect("Error generating Rsa Key");
        let new_public_key = RsaPublicKey::from(&new_private_key);    
        let pem_public_key = new_public_key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF).unwrap();
        let pem_private_key = new_private_key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF).unwrap();
        let pem_private_key_str = String::from(AsRef::<str>::as_ref(&pem_private_key));
        let wallet = Wallet{
            user_name,
            priv_key_pem:pem_private_key_str,
            pub_key_pem:pem_public_key
        };
        wallet

    }

    /// return the user name
    pub fn get_user_name(&self) -> String {
        return self.user_name.clone();
    }

    /// return the user id (transformed from the public key)
    pub fn get_user_id(&self) -> String {
        // Get user id from the public key by changing the format (strip off the first and last lines and join the middle lines)
        // Pub key format:  "-----BEGIN RSA PUBLIC KEY-----\nMDgCMQCqrJ1yIJ7cDQIdTuS+4CkKn/tQPN7bZFbbGCBhvjQxs71f6Vu+sD9eh8JG\npfiZSckCAwEAAQ==\n-----END RSA PUBLIC KEY-----\n"
        // user_id format:  "MDgCMQCqrJ1yIJ7cDQIdTuS+4CkKn/tQPN7bZFbbGCBhvjQxs71f6Vu+sD9eh8JGpfiZSckCAwEAAQ=="
        let public_key = self.pub_key_pem.trim_start_matches("-----BEGIN RSA PUBLIC KEY-----\n").trim_end_matches("\n-----END RSA PUBLIC KEY-----\n");
        let user_id = public_key.replace("\n","");
        //println!("{}", user_id);
        user_id
    }

    /// Sign a message using the private key and return the signature as a Base64 encoded string.
    /// To check if your implementation is correct, you can validate it using the `verify` method below in the unit tests.
    pub fn sign(&self, message: &str) -> String {
        // Sign the message with the private key, and return the signature in Base64 format
        let private_key = RsaPrivateKey::from_pkcs1_pem(&self.priv_key_pem).unwrap();
        let signing_key = SigningKey::<Sha256>::new(private_key);
        let mut rng = rand::thread_rng();
        let signature = signing_key.sign_with_rng(&mut rng, message.as_bytes());
        let encoded = Base64::encode_string(signature.as_bytes());
        encoded
    }

    /// Verify a signature using the public key. The signature is a string in Base64 format.
    pub fn verify(&self, message: &str, signature64: &str) -> bool {
        let public_key = rsa::RsaPublicKey::from_pkcs1_pem(&self.pub_key_pem).unwrap();
        let verifying_key = VerifyingKey::<Sha256>::new(public_key);

        let signature = Base64::decode_vec(&signature64).unwrap();
        let verify_signature = Signature::from_bytes(&signature).unwrap();
        let verify_result = verifying_key.verify(message.as_bytes(), &verify_signature);
        return match verify_result {
            Ok(()) => true,
            Err(e) => {
                //println!("[Signature verification failed]: {}", e);
                false
            }
        }
    }
}

