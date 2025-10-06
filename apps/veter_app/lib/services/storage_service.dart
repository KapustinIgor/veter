import 'package:flutter/foundation.dart';
import 'package:sqflite/sqflite.dart';
import 'package:path/path.dart';
import '../models/message.dart';

/// Storage service for local database operations
class StorageService extends ChangeNotifier {
  Database? _database;
  bool _isInitialized = false;

  bool get isInitialized => _isInitialized;

  Future<void> initialize() async {
    try {
      final databasePath = await getDatabasesPath();
      final path = join(databasePath, 'veter.db');

      _database = await openDatabase(
        path,
        version: 1,
        onCreate: _createTables,
        onUpgrade: _upgradeDatabase,
      );

      _isInitialized = true;
      notifyListeners();
    } catch (e) {
      debugPrint('Failed to initialize storage service: $e');
      rethrow;
    }
  }

  Future<void> _createTables(Database db, int version) async {
    // Create users table
    await db.execute('''
      CREATE TABLE users (
        id TEXT PRIMARY KEY,
        username TEXT UNIQUE NOT NULL,
        display_name TEXT NOT NULL,
        avatar_url TEXT,
        created_at INTEGER NOT NULL
      )
    ''');

    // Create rooms table
    await db.execute('''
      CREATE TABLE rooms (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        description TEXT,
        room_type TEXT NOT NULL,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
      )
    ''');

    // Create room_members table
    await db.execute('''
      CREATE TABLE room_members (
        room_id TEXT NOT NULL,
        user_id TEXT NOT NULL,
        joined_at INTEGER NOT NULL,
        PRIMARY KEY (room_id, user_id),
        FOREIGN KEY (room_id) REFERENCES rooms (id),
        FOREIGN KEY (user_id) REFERENCES users (id)
      )
    ''');

    // Create messages table
    await db.execute('''
      CREATE TABLE messages (
        id TEXT PRIMARY KEY,
        room_id TEXT NOT NULL,
        sender_id TEXT NOT NULL,
        sender_device_id TEXT NOT NULL,
        content_type TEXT NOT NULL,
        content_data TEXT NOT NULL,
        created_at INTEGER NOT NULL,
        edited_at INTEGER,
        reply_to TEXT,
        FOREIGN KEY (room_id) REFERENCES rooms (id),
        FOREIGN KEY (sender_id) REFERENCES users (id),
        FOREIGN KEY (sender_device_id) REFERENCES devices (id)
      )
    ''');

    // Create FTS5 virtual table for full-text search
    await db.execute('''
      CREATE VIRTUAL TABLE messages_fts USING fts5(
        content_data,
        content='messages',
        content_rowid='rowid'
      )
    ''');
  }

  Future<void> _upgradeDatabase(Database db, int oldVersion, int newVersion) async {
    // Handle database upgrades here
  }

  /// Store a message
  Future<void> storeMessage(Message message) async {
    if (!_isInitialized) {
      throw Exception('Storage service not initialized');
    }

    await _database!.insert(
      'messages',
      {
        'id': message.id,
        'room_id': message.roomId,
        'sender_id': message.senderId,
        'sender_device_id': message.senderDeviceId,
        'content_type': message.content.type.name,
        'content_data': _serializeMessageContent(message.content),
        'created_at': message.createdAt.millisecondsSinceEpoch,
        'edited_at': message.editedAt?.millisecondsSinceEpoch,
        'reply_to': message.replyTo,
      },
      conflictAlgorithm: ConflictAlgorithm.replace,
    );

    // Update FTS index
    await _database!.rawInsert('''
      INSERT INTO messages_fts (rowid, content_data)
      VALUES (last_insert_rowid(), ?)
    ''', [_serializeMessageContent(message.content)]);
  }

  /// Get messages for a room
  Future<List<Message>> getMessagesForRoom(String roomId, {int limit = 50, int offset = 0}) async {
    if (!_isInitialized) {
      throw Exception('Storage service not initialized');
    }

    final results = await _database!.query(
      'messages',
      where: 'room_id = ?',
      whereArgs: [roomId],
      orderBy: 'created_at DESC',
      limit: limit,
      offset: offset,
    );

    return results.map((row) => _deserializeMessage(row)).toList();
  }

  /// Search messages using full-text search
  Future<List<Message>> searchMessages(String query, {int limit = 20}) async {
    if (!_isInitialized) {
      throw Exception('Storage service not initialized');
    }

    final results = await _database!.rawQuery('''
      SELECT m.*
      FROM messages m
      JOIN messages_fts fts ON m.rowid = fts.rowid
      WHERE messages_fts MATCH ?
      ORDER BY fts.rank
      LIMIT ?
    ''', [query, limit]);

    return results.map((row) => _deserializeMessage(row)).toList();
  }

  /// Store a room
  Future<void> storeRoom(Room room) async {
    if (!_isInitialized) {
      throw Exception('Storage service not initialized');
    }

    await _database!.insert(
      'rooms',
      {
        'id': room.id,
        'name': room.name,
        'description': room.description,
        'room_type': room.type.name,
        'created_at': room.createdAt.millisecondsSinceEpoch,
        'updated_at': room.updatedAt.millisecondsSinceEpoch,
      },
      conflictAlgorithm: ConflictAlgorithm.replace,
    );
  }

  /// Get all rooms
  Future<List<Room>> getRooms() async {
    if (!_isInitialized) {
      throw Exception('Storage service not initialized');
    }

    final results = await _database!.query('rooms', orderBy: 'updated_at DESC');
    return results.map((row) => _deserializeRoom(row)).toList();
  }

  /// Store a user
  Future<void> storeUser(User user) async {
    if (!_isInitialized) {
      throw Exception('Storage service not initialized');
    }

    await _database!.insert(
      'users',
      {
        'id': user.id,
        'username': user.username,
        'display_name': user.displayName,
        'avatar_url': user.avatarUrl,
        'created_at': user.createdAt.millisecondsSinceEpoch,
      },
      conflictAlgorithm: ConflictAlgorithm.replace,
    );
  }

  /// Get user by ID
  Future<User?> getUser(String userId) async {
    if (!_isInitialized) {
      throw Exception('Storage service not initialized');
    }

    final results = await _database!.query(
      'users',
      where: 'id = ?',
      whereArgs: [userId],
      limit: 1,
    );

    if (results.isEmpty) return null;
    return _deserializeUser(results.first);
  }

  String _serializeMessageContent(MessageContent content) {
    switch (content.type) {
      case MessageContentType.text:
        return content.text ?? '';
      case MessageContentType.file:
        return '${content.fileName}|${content.mimeType}|${content.fileSize}|${content.fileUrl}';
      case MessageContentType.image:
        return '${content.imageUrl}|${content.imageWidth}|${content.imageHeight}';
      case MessageContentType.reaction:
        return '${content.emoji}|${content.targetMessageId}';
      case MessageContentType.system:
        return content.text ?? '';
    }
  }

  MessageContent _deserializeMessageContent(String contentType, String contentData) {
    switch (MessageContentType.values.firstWhere((e) => e.name == contentType)) {
      case MessageContentType.text:
        return MessageContent.text(contentData);
      case MessageContentType.file:
        final parts = contentData.split('|');
        return MessageContent.file(
          fileName: parts[0],
          mimeType: parts[1],
          fileSize: int.parse(parts[2]),
          fileUrl: parts[3],
        );
      case MessageContentType.image:
        final parts = contentData.split('|');
        return MessageContent.image(
          imageUrl: parts[0],
          imageWidth: int.parse(parts[1]),
          imageHeight: int.parse(parts[2]),
        );
      case MessageContentType.reaction:
        final parts = contentData.split('|');
        return MessageContent.reaction(
          emoji: parts[0],
          targetMessageId: parts[1],
        );
      case MessageContentType.system:
        return MessageContent.system(contentData);
    }
  }

  Message _deserializeMessage(Map<String, dynamic> row) {
    return Message(
      id: row['id'],
      roomId: row['room_id'],
      senderId: row['sender_id'],
      senderDeviceId: row['sender_device_id'],
      content: _deserializeMessageContent(row['content_type'], row['content_data']),
      createdAt: DateTime.fromMillisecondsSinceEpoch(row['created_at']),
      editedAt: row['edited_at'] != null 
          ? DateTime.fromMillisecondsSinceEpoch(row['edited_at'])
          : null,
      replyTo: row['reply_to'],
    );
  }

  Room _deserializeRoom(Map<String, dynamic> row) {
    return Room(
      id: row['id'],
      name: row['name'],
      description: row['description'],
      type: RoomType.values.firstWhere((e) => e.name == row['room_type']),
      members: [], // TODO: Load members separately
      createdAt: DateTime.fromMillisecondsSinceEpoch(row['created_at']),
      updatedAt: DateTime.fromMillisecondsSinceEpoch(row['updated_at']),
    );
  }

  User _deserializeUser(Map<String, dynamic> row) {
    return User(
      id: row['id'],
      username: row['username'],
      displayName: row['display_name'],
      avatarUrl: row['avatar_url'],
      createdAt: DateTime.fromMillisecondsSinceEpoch(row['created_at']),
    );
  }

  @override
  void dispose() {
    _database?.close();
    super.dispose();
  }
}
