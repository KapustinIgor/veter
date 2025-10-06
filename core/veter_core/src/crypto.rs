//! Cryptographic operations for Veter

use crate::{VeterError, Result, models::*};
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::Aead;
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce, KeyInit as ChaChaKeyInit};
use rand::RngCore;
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use std::collections::HashMap;

/// Cryptographic operations manager
pub struct CryptoManager {
    identity_key: Vec<u8>,
    device_id: DeviceId,
    sessions: HashMap<RoomId, Session>,
}

impl CryptoManager {
    /// Create a new crypto manager
    pub fn new(identity_key: Vec<u8>, device_id: DeviceId) -> Self {
        Self {
            identity_key,
            device_id,
            sessions: HashMap::new(),
        }
    }

    /// Generate a new identity key pair
    pub fn generate_identity_keypair() -> Result<(Vec<u8>, Vec<u8>)> {
        // TODO: Use libsignal-protocol for proper key generation
        // For now, generate random keys as placeholder
        let mut private_key = vec![0u8; 32];
        let mut public_key = vec![0u8; 32];
        
        rand::thread_rng().fill_bytes(&mut private_key);
        rand::thread_rng().fill_bytes(&mut public_key);
        
        Ok((private_key, public_key))
    }

    /// Encrypt message content using AES-GCM
    pub fn encrypt_message(&self, content: &[u8], room_id: RoomId) -> Result<Vec<u8>> {
        // Generate random key and nonce for this message
        let mut key_bytes = [0u8; 32];
        let mut nonce_bytes = [0u8; 12];
        
        rand::thread_rng().fill_bytes(&mut key_bytes);
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        
        let key = Key::from_slice(&key_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let cipher = Aes256Gcm::new(key);
        let ciphertext = cipher.encrypt(nonce, content)
            .map_err(|e| VeterError::Crypto(format!("Encryption failed: {}", e)))?;
        
        // TODO: Encrypt the key using libsignal session
        // For now, prepend key and nonce (insecure!)
        let mut result = Vec::new();
        result.extend_from_slice(&key_bytes);
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    /// Decrypt message content using AES-GCM
    pub fn decrypt_message(&self, encrypted: &[u8], room_id: RoomId) -> Result<Vec<u8>> {
        if encrypted.len() < 44 { // 32 + 12 = key + nonce
            return Err(VeterError::Crypto("Invalid encrypted message format".to_string()));
        }
        
        let key_bytes = &encrypted[0..32];
        let nonce_bytes = &encrypted[32..44];
        let ciphertext = &encrypted[44..];
        
        let key = Key::from_slice(key_bytes);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let cipher = Aes256Gcm::new(key);
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| VeterError::Crypto(format!("Decryption failed: {}", e)))?;
        
        Ok(plaintext)
    }

    /// Encrypt file content using ChaCha20-Poly1305
    pub fn encrypt_file(&self, content: &[u8]) -> Result<Vec<u8>> {
        let mut key_bytes = [0u8; 32];
        let mut nonce_bytes = [0u8; 12];
        
        rand::thread_rng().fill_bytes(&mut key_bytes);
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        
        let key = ChaChaKey::from_slice(&key_bytes);
        let nonce = ChaChaNonce::from_slice(&nonce_bytes);
        
        let cipher = ChaCha20Poly1305::new(key);
        let ciphertext = cipher.encrypt(nonce, content)
            .map_err(|e| VeterError::Crypto(format!("File encryption failed: {}", e)))?;
        
        // Prepend key and nonce (TODO: encrypt with envelope key)
        let mut result = Vec::new();
        result.extend_from_slice(&key_bytes);
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    /// Decrypt file content using ChaCha20-Poly1305
    pub fn decrypt_file(&self, encrypted: &[u8]) -> Result<Vec<u8>> {
        if encrypted.len() < 44 {
            return Err(VeterError::Crypto("Invalid encrypted file format".to_string()));
        }
        
        let key_bytes = &encrypted[0..32];
        let nonce_bytes = &encrypted[32..44];
        let ciphertext = &encrypted[44..];
        
        let key = ChaChaKey::from_slice(key_bytes);
        let nonce = ChaChaNonce::from_slice(nonce_bytes);
        
        let cipher = ChaCha20Poly1305::new(key);
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| VeterError::Crypto(format!("File decryption failed: {}", e)))?;
        
        Ok(plaintext)
    }

    /// Generate HMAC for message authentication
    pub fn generate_hmac(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut mac = Hmac::<Sha256>::new_from_slice(&self.identity_key)
            .map_err(|e| VeterError::Crypto(format!("HMAC creation failed: {}", e)))?;
        
        mac.update(data);
        Ok(mac.finalize().into_bytes().to_vec())
    }

    /// Verify HMAC for message authentication
    pub fn verify_hmac(&self, data: &[u8], mac: &[u8]) -> Result<bool> {
        let mut expected_mac = Hmac::<Sha256>::new_from_slice(&self.identity_key)
            .map_err(|e| VeterError::Crypto(format!("HMAC creation failed: {}", e)))?;
        
        expected_mac.update(data);
        let expected = expected_mac.finalize().into_bytes();
        
        Ok(expected.as_slice() == mac)
    }

    /// Initialize session for a room
    pub fn init_session(&mut self, room_id: RoomId, session_data: Vec<u8>) -> Result<()> {
        let session = Session {
            room_id,
            device_id: self.device_id,
            session_data,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        self.sessions.insert(room_id, session);
        Ok(())
    }

    /// Get session for a room
    pub fn get_session(&self, room_id: RoomId) -> Option<&Session> {
        self.sessions.get(&room_id)
    }
}
