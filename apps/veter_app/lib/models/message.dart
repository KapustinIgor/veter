import 'package:uuid/uuid.dart';

/// Message content types
enum MessageContentType {
  text,
  file,
  image,
  reaction,
  system,
}

/// Message content
class MessageContent {
  final MessageContentType type;
  final String? text;
  final String? fileName;
  final String? mimeType;
  final int? fileSize;
  final String? fileUrl;
  final String? imageUrl;
  final int? imageWidth;
  final int? imageHeight;
  final String? emoji;
  final String? targetMessageId;

  const MessageContent.text(String this.text)
      : type = MessageContentType.text,
        fileName = null,
        mimeType = null,
        fileSize = null,
        fileUrl = null,
        imageUrl = null,
        imageWidth = null,
        imageHeight = null,
        emoji = null,
        targetMessageId = null;

  const MessageContent.file({
    required String this.fileName,
    required String this.mimeType,
    required int this.fileSize,
    required String this.fileUrl,
  })  : type = MessageContentType.file,
        text = null,
        imageUrl = null,
        imageWidth = null,
        imageHeight = null,
        emoji = null,
        targetMessageId = null;

  const MessageContent.image({
    required String this.imageUrl,
    required int this.imageWidth,
    required int this.imageHeight,
  })  : type = MessageContentType.image,
        text = null,
        fileName = null,
        mimeType = null,
        fileSize = null,
        fileUrl = null,
        emoji = null,
        targetMessageId = null;

  const MessageContent.reaction({
    required String this.emoji,
    required String this.targetMessageId,
  })  : type = MessageContentType.reaction,
        text = null,
        fileName = null,
        mimeType = null,
        fileSize = null,
        fileUrl = null,
        imageUrl = null,
        imageWidth = null,
        imageHeight = null;

  const MessageContent.system(String this.text)
      : type = MessageContentType.system,
        fileName = null,
        mimeType = null,
        fileSize = null,
        fileUrl = null,
        imageUrl = null,
        imageWidth = null,
        imageHeight = null,
        emoji = null,
        targetMessageId = null;
}

/// Message model
class Message {
  final String id;
  final String roomId;
  final String senderId;
  final String senderDeviceId;
  final MessageContent content;
  final DateTime createdAt;
  final DateTime? editedAt;
  final String? replyTo;

  const Message({
    required this.id,
    required this.roomId,
    required this.senderId,
    required this.senderDeviceId,
    required this.content,
    required this.createdAt,
    this.editedAt,
    this.replyTo,
  });

  Message copyWith({
    String? id,
    String? roomId,
    String? senderId,
    String? senderDeviceId,
    MessageContent? content,
    DateTime? createdAt,
    DateTime? editedAt,
    String? replyTo,
  }) {
    return Message(
      id: id ?? this.id,
      roomId: roomId ?? this.roomId,
      senderId: senderId ?? this.senderId,
      senderDeviceId: senderDeviceId ?? this.senderDeviceId,
      content: content ?? this.content,
      createdAt: createdAt ?? this.createdAt,
      editedAt: editedAt ?? this.editedAt,
      replyTo: replyTo ?? this.replyTo,
    );
  }

  factory Message.create({
    required String roomId,
    required String senderId,
    required String senderDeviceId,
    required MessageContent content,
    String? replyTo,
  }) {
    return Message(
      id: const Uuid().v4(),
      roomId: roomId,
      senderId: senderId,
      senderDeviceId: senderDeviceId,
      content: content,
      createdAt: DateTime.now(),
      replyTo: replyTo,
    );
  }
}

/// Room model
class Room {
  final String id;
  final String name;
  final String? description;
  final RoomType type;
  final List<String> members;
  final DateTime createdAt;
  final DateTime updatedAt;

  const Room({
    required this.id,
    required this.name,
    this.description,
    required this.type,
    required this.members,
    required this.createdAt,
    required this.updatedAt,
  });
}

/// Room types
enum RoomType {
  direct,
  group,
  channel,
}

/// User model
class User {
  final String id;
  final String username;
  final String displayName;
  final String? avatarUrl;
  final DateTime createdAt;

  const User({
    required this.id,
    required this.username,
    required this.displayName,
    this.avatarUrl,
    required this.createdAt,
  });
}

/// Device model
class Device {
  final String id;
  final String userId;
  final String name;
  final Platform platform;
  final DateTime createdAt;
  final DateTime lastSeen;

  const Device({
    required this.id,
    required this.userId,
    required this.name,
    required this.platform,
    required this.createdAt,
    required this.lastSeen,
  });
}

/// Platform types
enum Platform {
  ios,
  android,
  macos,
  windows,
  linux,
  web,
}
