// SPDX-License-Identifier: GNU GENERAL PUBLIC LICENSE Version 3
//
// Copyleft (c) 2024 James Wong. This file is part of James Wong.
// is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// James Wong is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with James Wong.  If not, see <https://www.gnu.org/licenses/>.
//
// IMPORTANT: Any software that fully or partially contains or uses materials
// covered by this license must also be released under the GNU GPL license.
// This includes modifications and derived works.

extern crate openssl;

use openssl::rsa::{ Rsa, Padding };
use openssl::pkey::Private;
use openssl::sha::sha256;
use base64::{ engine::general_purpose, Engine as _ };
use std::error::Error;

pub struct RSACipher {
    private_key: Rsa<Private>,
}

impl RSACipher {
    pub fn new(bits: u32) -> Result<Self, Box<dyn Error>> {
        let private_key = Rsa::generate(bits)?;
        Ok(Self { private_key })
    }

    pub fn from_base64(base64_private_key: &str) -> Result<Self, Box<dyn Error>> {
        let pem = general_purpose::STANDARD.decode(base64_private_key)?;
        let private_key = Rsa::private_key_from_pem(&pem)?;
        Ok(Self { private_key })
    }

    pub fn from_hex(hex_private_key: &str) -> Result<Self, Box<dyn Error>> {
        let pem = hex::decode(hex_private_key)?;
        let private_key = Rsa::private_key_from_pem(&pem)?;
        Ok(Self { private_key })
    }

    pub fn get_base64_public_key(&self) -> Result<String, Box<dyn Error>> {
        let pem = self.private_key.public_key_to_pem()?;
        Ok(general_purpose::STANDARD.encode(pem))
    }

    pub fn get_hex_public_key(&self) -> Result<String, Box<dyn Error>> {
        let pem = self.private_key.public_key_to_pem()?;
        Ok(hex::encode(pem))
    }

    pub fn get_base64_private_key(&self) -> Result<String, Box<dyn Error>> {
        let pem = self.private_key.private_key_to_pem()?;
        Ok(general_purpose::STANDARD.encode(pem))
    }

    pub fn get_hex_private_key(&self) -> Result<String, Box<dyn Error>> {
        let pem = self.private_key.private_key_to_pem()?;
        Ok(hex::encode(pem))
    }

    // Public key encryption.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buf: Vec<u8> = vec![0; self.private_key.size() as usize];
        let len = self.private_key.public_encrypt(plaintext, &mut buf, Padding::PKCS1)?;
        buf.truncate(len);
        Ok(buf)
    }

    // Private key decryption.
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buf: Vec<u8> = vec![0; self.private_key.size() as usize];
        let len = self.private_key.private_decrypt(ciphertext, &mut buf, Padding::PKCS1)?;
        buf.truncate(len);
        Ok(buf)
    }

    // Private key decryption with base64.
    pub fn decrypt_from_base64(&self, base64_ciphertext: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buf: Vec<u8> = vec![0; self.private_key.size() as usize];
        match base64_decode(base64_ciphertext) {
            std::result::Result::Ok(ciphertext) => {
                let len = self.private_key.private_decrypt(&ciphertext, &mut buf, Padding::PKCS1)?;
                buf.truncate(len);
                Ok(buf)
            }
            Err(e) => Err(Box::new(e)),
        }
    }

    // Private key signing.
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let hash = sha256(message);
        let mut buf: Vec<u8> = vec![0; self.private_key.size() as usize];
        let len = self.private_key.private_encrypt(&hash, &mut buf, Padding::PKCS1)?;
        buf.truncate(len);
        Ok(buf)
    }

    // Public key verification for private key signature.
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<(), Box<dyn Error>> {
        let hash = sha256(message);
        let mut buf: Vec<u8> = vec![0; self.private_key.size() as usize];
        let len = self.private_key.public_decrypt(signature, &mut buf, Padding::PKCS1)?;
        buf.truncate(len);
        if buf == hash {
            Ok(())
        } else {
            Err("Verification failed".into())
        }
    }
}

pub fn base64_encode(input: &[u8]) -> String {
    general_purpose::STANDARD.encode(input)
}

pub fn base64_decode(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
    general_purpose::STANDARD.decode(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsa_cipher() {
        let cipher = RSACipher::new(2048).unwrap();
        let base64_pubkey = cipher.get_base64_public_key().unwrap();
        assert!(!base64_pubkey.is_empty());

        let hex_pubkey = cipher.get_hex_public_key().unwrap();
        assert!(!hex_pubkey.is_empty());

        let base64_privkey = cipher.get_base64_private_key().unwrap();
        assert!(!base64_privkey.is_empty());

        let hex_privkey = cipher.get_hex_private_key().unwrap();
        assert!(!hex_privkey.is_empty());

        // Test reconstruction from base64 private key
        let reconstructed_cipher = RSACipher::from_base64(&base64_privkey).unwrap();
        let reconstructed_pubkey = reconstructed_cipher.get_base64_public_key().unwrap();
        assert_eq!(base64_pubkey, reconstructed_pubkey);

        // Test reconstruction from hex private key
        let reconstructed_cipher = RSACipher::from_hex(&hex_privkey).unwrap();
        let reconstructed_pubkey = reconstructed_cipher.get_hex_public_key().unwrap();
        assert_eq!(hex_pubkey, reconstructed_pubkey);

        // Test encryption and decryption
        let plaintext = b"Hello, World!";
        let encrypted = cipher.encrypt(plaintext).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(plaintext, &decrypted[..]);

        // Test signing and verification
        let message = b"Important message";
        let signature = cipher.sign(message).unwrap();
        assert!(cipher.verify(message, &signature).is_ok());

        // Test failed verification with wrong message
        let wrong_message = b"Wrong message";
        assert!(cipher.verify(wrong_message, &signature).is_err());
    }

    #[test]
    fn test_base64() {
        let original = b"Hello, World!";
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(original, decoded.as_slice());
    }
}
