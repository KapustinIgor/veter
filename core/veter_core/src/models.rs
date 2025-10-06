//! Core data models for Veter

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Unique identifier for a device
pub type DeviceId = Uuid;

/// Unique identifier for a user
pub type UserId = Uuid;

/// Unique identifier for a room/conversation
pub type RoomId = Uuid;

/// Unique identifier for a message
pub type MessageId = Uuid;

/// Device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: DeviceId,
    pub user_id: UserId,
    pub name: String,
    pub platform: Platform,
    pub public_key: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

/// Platform types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Platform {
    Ios,
    Android,
    Macos,
    Windows,
    Linux,
    Web,
}

/// User profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Room/conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: RoomId,
    pub name: String,
    pub description: Option<String>,
    pub room_type: RoomType,
    pub members: Vec<UserId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Room types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoomType {
    Direct,  // 1:1 conversation
    Group,   // Group conversation
    Channel, // Broadcast channel
}

/// Message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub room_id: RoomId,
    pub sender_id: UserId,
    pub sender_device_id: DeviceId,
    pub content: MessageContent,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub reply_to: Option<MessageId>,
}

/// Message content types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    File {
        name: String,
        mime_type: String,
        size: u64,
        url: String,
    },
    Image {
        url: String,
        width: u32,
        height: u32,
    },
    Reaction {
        emoji: String,
        target_message_id: MessageId,
    },
    System(String), // System messages (user joined, etc.)
}

/// Encrypted message payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub id: MessageId,
    pub room_id: RoomId,
    pub sender_device_id: DeviceId,
    pub payload: Vec<u8>, // E2EE encrypted content
    pub timestamp: DateTime<Utc>,
}

/// Key material for encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMaterial {
    pub identity_key: Vec<u8>,
    pub signed_prekey: Vec<u8>,
    pub one_time_prekeys: Vec<Vec<u8>>,
}

/// Session state for a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub room_id: RoomId,
    pub device_id: DeviceId,
    pub session_data: Vec<u8>, // libsignal session state
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
