import 'dart:convert';

import 'package:flutter_secure_storage/flutter_secure_storage.dart';

import '../model/auth_session.dart';

class AuthSessionStorage {
  AuthSessionStorage({FlutterSecureStorage? secureStorage})
      : _secureStorage = secureStorage ?? const FlutterSecureStorage();

  static const String _sessionKey = 'auth_session_v1';

  final FlutterSecureStorage _secureStorage;

  Future<AuthSession?> read() async {
    final raw = await _secureStorage.read(key: _sessionKey);
    if (raw == null || raw.isEmpty) {
      return null;
    }

    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic>) {
        await clear();
        return null;
      }

      final session = AuthSession.fromStorageJson(decoded);
      if (!session.isUsable) {
        await clear();
        return null;
      }
      return session;
    } catch (_) {
      await clear();
      return null;
    }
  }

  Future<void> write(AuthSession session) {
    return _secureStorage.write(
      key: _sessionKey,
      value: jsonEncode(session.toStorageJson()),
    );
  }

  Future<void> clear() {
    return _secureStorage.delete(key: _sessionKey);
  }
}
