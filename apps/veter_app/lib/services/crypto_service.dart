import 'package:flutter/foundation.dart';
import 'package:crypto/crypto.dart';
import 'dart:convert';
import 'dart:typed_data';

/// Crypto service for encryption/decryption
class CryptoService extends ChangeNotifier {
  bool _isInitialized = false;
  Uint8List? _identityKey;
  Uint8List? _deviceKey;

  bool get isInitialized => _isInitialized;

  Future<void> initialize() async {
    try {
      // Generate identity key (in real implementation, this would be hardware-backed)
      _identityKey = _generateRandomBytes(32);
      _deviceKey = _generateRandomBytes(32);
      
      _isInitialized = true;
      notifyListeners();
    } catch (e) {
      debugPrint('Failed to initialize crypto service: $e');
      rethrow;
    }
  }

  /// Encrypt message content
  Future<Uint8List> encryptMessage(String content) async {
    if (!_isInitialized) {
      throw Exception('Crypto service not initialized');
    }

    // Simple XOR encryption for demo (replace with proper AES-GCM)
    final contentBytes = utf8.encode(content);
    final encrypted = Uint8List(contentBytes.length);
    
    for (int i = 0; i < contentBytes.length; i++) {
      encrypted[i] = contentBytes[i] ^ _identityKey![i % _identityKey!.length];
    }
    
    return encrypted;
  }

  /// Decrypt message content
  Future<String> decryptMessage(Uint8List encrypted) async {
    if (!_isInitialized) {
      throw Exception('Crypto service not initialized');
    }

    // Simple XOR decryption for demo (replace with proper AES-GCM)
    final decrypted = Uint8List(encrypted.length);
    
    for (int i = 0; i < encrypted.length; i++) {
      decrypted[i] = encrypted[i] ^ _identityKey![i % _identityKey!.length];
    }
    
    return utf8.decode(decrypted);
  }

  /// Encrypt file content
  Future<Uint8List> encryptFile(Uint8List fileData) async {
    if (!_isInitialized) {
      throw Exception('Crypto service not initialized');
    }

    // Simple XOR encryption for demo (replace with proper ChaCha20-Poly1305)
    final encrypted = Uint8List(fileData.length);
    
    for (int i = 0; i < fileData.length; i++) {
      encrypted[i] = fileData[i] ^ _deviceKey![i % _deviceKey!.length];
    }
    
    return encrypted;
  }

  /// Decrypt file content
  Future<Uint8List> decryptFile(Uint8List encrypted) async {
    if (!_isInitialized) {
      throw Exception('Crypto service not initialized');
    }

    // Simple XOR decryption for demo (replace with proper ChaCha20-Poly1305)
    final decrypted = Uint8List(encrypted.length);
    
    for (int i = 0; i < encrypted.length; i++) {
      decrypted[i] = encrypted[i] ^ _deviceKey![i % _deviceKey!.length];
    }
    
    return decrypted;
  }

  /// Generate HMAC for message authentication
  Future<String> generateHmac(String data) async {
    if (!_isInitialized) {
      throw Exception('Crypto service not initialized');
    }

    final key = utf8.encode('hmac-key'); // In real implementation, use proper key
    final bytes = utf8.encode(data);
    final hmac = Hmac(sha256, key);
    final digest = hmac.convert(bytes);
    
    return digest.toString();
  }

  /// Verify HMAC
  Future<bool> verifyHmac(String data, String mac) async {
    final expectedMac = await generateHmac(data);
    return expectedMac == mac;
  }

  Uint8List _generateRandomBytes(int length) {
    final random = DateTime.now().millisecondsSinceEpoch;
    final bytes = Uint8List(length);
    
    for (int i = 0; i < length; i++) {
      bytes[i] = (random >> (i % 32)) & 0xFF;
    }
    
    return bytes;
  }
}
