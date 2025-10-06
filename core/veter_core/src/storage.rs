//! Local storage and database operations

use crate::{VeterError, Result, models::*};
use sqlx::{SqlitePool, Row};
use std::path::Path;
use uuid::Uuid;

/// Database manager for local storage
pub struct StorageManager {
    pool: SqlitePool,
}

impl StorageManager {
    /// Create a new storage manager with encrypted SQLite database
    pub async fn new(db_path: &Path, password: &str) -> Result<Self> {
        // TODO: Use SQLCipher for encryption
        // For now, use regular SQLite
        let database_url = format!("sqlite://{}", db_path.display());
        
        let pool = SqlitePool::connect(&database_url)
            .await
            .map_err(|e| VeterError::Database(format!("Failed to connect to database: {}", e)))?;
        
        let manager = Self { pool };
        manager.init_schema().await?;
        
        Ok(manager)
    }

    /// Initialize database schema
    async fn init_schema(&self) -> Result<()> {
        // Create users table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                display_name TEXT NOT NULL,
                avatar_url TEXT,
                created_at TEXT NOT NULL
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| VeterError::Database(format!("Failed to create users table: {}", e)))?;

        // Create devices table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS devices (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                name TEXT NOT NULL,
                platform TEXT NOT NULL,
                public_key BLOB NOT NULL,
                created_at TEXT NOT NULL,
                last_seen TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users (id)
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| VeterError::Database(format!("Failed to create devices table: {}", e)))?;

        // Create rooms table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS rooms (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                room_type TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| VeterError::Database(format!("Failed to create rooms table: {}", e)))?;

        // Create room_members table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS room_members (
                room_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                joined_at TEXT NOT NULL,
                PRIMARY KEY (room_id, user_id),
                FOREIGN KEY (room_id) REFERENCES rooms (id),
                FOREIGN KEY (user_id) REFERENCES users (id)
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| VeterError::Database(format!("Failed to create room_members table: {}", e)))?;

        // Create messages table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                room_id TEXT NOT NULL,
                sender_id TEXT NOT NULL,
                sender_device_id TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                edited_at TEXT,
                reply_to TEXT,
                FOREIGN KEY (room_id) REFERENCES rooms (id),
                FOREIGN KEY (sender_id) REFERENCES users (id),
                FOREIGN KEY (sender_device_id) REFERENCES devices (id)
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| VeterError::Database(format!("Failed to create messages table: {}", e)))?;

        // Create sessions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                room_id TEXT PRIMARY KEY,
                device_id TEXT NOT NULL,
                session_data BLOB NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (room_id) REFERENCES rooms (id),
                FOREIGN KEY (device_id) REFERENCES devices (id)
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| VeterError::Database(format!("Failed to create sessions table: {}", e)))?;

        // Create FTS5 virtual table for full-text search
        sqlx::query(
            r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
                content,
                content='messages',
                content_rowid='rowid'
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| VeterError::Database(format!("Failed to create FTS table: {}", e)))?;

        Ok(())
    }

    /// Store a user
    pub async fn store_user(&self, user: &User) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO users (id, username, display_name, avatar_url, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(&user.id.to_string())
        .bind(&user.username)
        .bind(&user.display_name)
        .bind(&user.avatar_url)
        .bind(&user.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| VeterError::Database(format!("Failed to store user: {}", e)))?;

        Ok(())
    }

    /// Get a user by ID
    pub async fn get_user(&self, user_id: &UserId) -> Result<Option<User>> {
        let row = sqlx::query(
            r#"
            SELECT id, username, display_name, avatar_url, created_at
            FROM users WHERE id = ?
            "#
        )
        .bind(&user_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| VeterError::Database(format!("Failed to get user: {}", e)))?;

        if let Some(row) = row {
            let user = User {
                id: Uuid::parse_str(&row.get::<String, _>("id"))
                    .map_err(|e| VeterError::Database(format!("Invalid user ID: {}", e)))?,
                username: row.get("username"),
                display_name: row.get("display_name"),
                avatar_url: row.get("avatar_url"),
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                    .map_err(|e| VeterError::Database(format!("Invalid timestamp: {}", e)))?
                    .with_timezone(&chrono::Utc),
            };
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    /// Store a message
    pub async fn store_message(&self, message: &Message) -> Result<()> {
        let content_json = serde_json::to_string(&message.content)
            .map_err(|e| VeterError::Serialization(format!("Failed to serialize message content: {}", e)))?;

        sqlx::query(
            r#"
            INSERT INTO messages (id, room_id, sender_id, sender_device_id, content, created_at, edited_at, reply_to)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&message.id.to_string())
        .bind(&message.room_id.to_string())
        .bind(&message.sender_id.to_string())
        .bind(&message.sender_device_id.to_string())
        .bind(&content_json)
        .bind(&message.created_at.to_rfc3339())
        .bind(&message.edited_at.map(|t| t.to_rfc3339()))
        .bind(&message.reply_to.map(|id| id.to_string()))
        .execute(&self.pool)
        .await
        .map_err(|e| VeterError::Database(format!("Failed to store message: {}", e)))?;

        // Update FTS index
        sqlx::query(
            r#"
            INSERT INTO messages_fts (rowid, content)
            VALUES (last_insert_rowid(), ?)
            "#
        )
        .bind(&content_json)
        .execute(&self.pool)
        .await
        .map_err(|e| VeterError::Database(format!("Failed to update FTS index: {}", e)))?;

        Ok(())
    }

    /// Get messages for a room
    pub async fn get_messages(&self, room_id: &RoomId, limit: i64, offset: i64) -> Result<Vec<Message>> {
        let rows = sqlx::query(
            r#"
            SELECT id, room_id, sender_id, sender_device_id, content, created_at, edited_at, reply_to
            FROM messages 
            WHERE room_id = ?
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#
        )
        .bind(&room_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| VeterError::Database(format!("Failed to get messages: {}", e)))?;

        let mut messages = Vec::new();
        for row in rows {
            let content: String = row.get("content");
            let message_content = serde_json::from_str(&content)
                .map_err(|e| VeterError::Serialization(format!("Failed to deserialize message content: {}", e)))?;

            let message = Message {
                id: Uuid::parse_str(&row.get::<String, _>("id"))
                    .map_err(|e| VeterError::Database(format!("Invalid message ID: {}", e)))?,
                room_id: Uuid::parse_str(&row.get::<String, _>("room_id"))
                    .map_err(|e| VeterError::Database(format!("Invalid room ID: {}", e)))?,
                sender_id: Uuid::parse_str(&row.get::<String, _>("sender_id"))
                    .map_err(|e| VeterError::Database(format!("Invalid sender ID: {}", e)))?,
                sender_device_id: Uuid::parse_str(&row.get::<String, _>("sender_device_id"))
                    .map_err(|e| VeterError::Database(format!("Invalid device ID: {}", e)))?,
                content: message_content,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                    .map_err(|e| VeterError::Database(format!("Invalid timestamp: {}", e)))?
                    .with_timezone(&chrono::Utc),
                edited_at: row.get::<Option<String>, _>("edited_at")
                    .map(|s| chrono::DateTime::parse_from_rfc3339(&s)
                        .map_err(|e| VeterError::Database(format!("Invalid timestamp: {}", e)))
                        .unwrap()
                        .with_timezone(&chrono::Utc)),
                reply_to: row.get::<Option<String>, _>("reply_to")
                    .map(|s| Uuid::parse_str(&s)
                        .map_err(|e| VeterError::Database(format!("Invalid reply ID: {}", e)))
                        .unwrap()),
            };
            messages.push(message);
        }

        Ok(messages)
    }

    /// Search messages using full-text search
    pub async fn search_messages(&self, query: &str, limit: i64) -> Result<Vec<Message>> {
        let rows = sqlx::query(
            r#"
            SELECT m.id, m.room_id, m.sender_id, m.sender_device_id, m.content, m.created_at, m.edited_at, m.reply_to
            FROM messages m
            JOIN messages_fts fts ON m.rowid = fts.rowid
            WHERE messages_fts MATCH ?
            ORDER BY fts.rank
            LIMIT ?
            "#
        )
        .bind(query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| VeterError::Database(format!("Failed to search messages: {}", e)))?;

        let mut messages = Vec::new();
        for row in rows {
            let content: String = row.get("content");
            let message_content = serde_json::from_str(&content)
                .map_err(|e| VeterError::Serialization(format!("Failed to deserialize message content: {}", e)))?;

            let message = Message {
                id: Uuid::parse_str(&row.get::<String, _>("id"))
                    .map_err(|e| VeterError::Database(format!("Invalid message ID: {}", e)))?,
                room_id: Uuid::parse_str(&row.get::<String, _>("room_id"))
                    .map_err(|e| VeterError::Database(format!("Invalid room ID: {}", e)))?,
                sender_id: Uuid::parse_str(&row.get::<String, _>("sender_id"))
                    .map_err(|e| VeterError::Database(format!("Invalid sender ID: {}", e)))?,
                sender_device_id: Uuid::parse_str(&row.get::<String, _>("sender_device_id"))
                    .map_err(|e| VeterError::Database(format!("Invalid device ID: {}", e)))?,
                content: message_content,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                    .map_err(|e| VeterError::Database(format!("Invalid timestamp: {}", e)))?
                    .with_timezone(&chrono::Utc),
                edited_at: row.get::<Option<String>, _>("edited_at")
                    .map(|s| chrono::DateTime::parse_from_rfc3339(&s)
                        .map_err(|e| VeterError::Database(format!("Invalid timestamp: {}", e)))
                        .unwrap()
                        .with_timezone(&chrono::Utc)),
                reply_to: row.get::<Option<String>, _>("reply_to")
                    .map(|s| Uuid::parse_str(&s)
                        .map_err(|e| VeterError::Database(format!("Invalid reply ID: {}", e)))
                        .unwrap()),
            };
            messages.push(message);
        }

        Ok(messages)
    }

    /// Store a session
    pub async fn store_session(&self, session: &Session) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO sessions (room_id, device_id, session_data, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(&session.room_id.to_string())
        .bind(&session.device_id.to_string())
        .bind(&session.session_data)
        .bind(&session.created_at.to_rfc3339())
        .bind(&session.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| VeterError::Database(format!("Failed to store session: {}", e)))?;

        Ok(())
    }

    /// Get a session
    pub async fn get_session(&self, room_id: &RoomId) -> Result<Option<Session>> {
        let row = sqlx::query(
            r#"
            SELECT room_id, device_id, session_data, created_at, updated_at
            FROM sessions WHERE room_id = ?
            "#
        )
        .bind(&room_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| VeterError::Database(format!("Failed to get session: {}", e)))?;

        if let Some(row) = row {
            let session = Session {
                room_id: Uuid::parse_str(&row.get::<String, _>("room_id"))
                    .map_err(|e| VeterError::Database(format!("Invalid room ID: {}", e)))?,
                device_id: Uuid::parse_str(&row.get::<String, _>("device_id"))
                    .map_err(|e| VeterError::Database(format!("Invalid device ID: {}", e)))?,
                session_data: row.get("session_data"),
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                    .map_err(|e| VeterError::Database(format!("Invalid timestamp: {}", e)))?
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))
                    .map_err(|e| VeterError::Database(format!("Invalid timestamp: {}", e)))?
                    .with_timezone(&chrono::Utc),
            };
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }
}
