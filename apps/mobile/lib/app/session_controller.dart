import 'package:flutter/foundation.dart';

import '../features/auth/data/auth_repository.dart';
import '../features/auth/data/auth_session_storage.dart';
import '../features/auth/model/auth_session.dart';

class SessionController extends ChangeNotifier {
  SessionController(this._storage);

  final AuthSessionStorage _storage;
  AuthSession? _session;
  bool _isBusy = false;
  bool _isRestoring = false;
  Future<bool>? _refreshFuture;

  AuthSession? get session => _session;
  String? get accessToken => _session?.accessToken;
  String? get refreshToken => _session?.refreshToken;
  bool get isAuthenticated => _session != null;
  bool get isBusy => _isBusy;
  bool get isRestoring => _isRestoring;

  Future<void> restore() async {
    _isRestoring = true;
    notifyListeners();
    try {
      _session = await _storage.read();
    } finally {
      _isRestoring = false;
      notifyListeners();
    }
  }

  Future<void> login({
    required AuthRepository repository,
    required String email,
    required String password,
  }) async {
    _setBusy(true);
    try {
      await _setSession(await repository.login(email: email, password: password));
    } finally {
      _setBusy(false);
    }
  }

  Future<void> register({
    required AuthRepository repository,
    required String email,
    required String password,
    required String displayName,
  }) async {
    _setBusy(true);
    try {
      await _setSession(
        await repository.register(
          email: email,
          password: password,
          displayName: displayName,
        ),
      );
    } finally {
      _setBusy(false);
    }
  }

  Future<void> logout() async {
    _session = null;
    await _storage.clear();
    notifyListeners();
  }

  Future<bool> refreshWith(
    Future<Map<String, dynamic>> Function(String refreshToken) loader,
  ) {
    final existing = _refreshFuture;
    if (existing != null) {
      return existing;
    }

    final currentRefreshToken = refreshToken;
    if (currentRefreshToken == null || currentRefreshToken.isEmpty) {
      return Future<bool>.value(false);
    }

    final future = _performRefresh(loader, currentRefreshToken);
    _refreshFuture = future;
    future.whenComplete(() => _refreshFuture = null);
    return future;
  }

  Future<bool> _performRefresh(
    Future<Map<String, dynamic>> Function(String refreshToken) loader,
    String currentRefreshToken,
  ) async {
    try {
      final payload = await loader(currentRefreshToken);
      await _setSession(AuthSession.fromJson(payload));
      return _session?.accessToken.isNotEmpty == true;
    } catch (_) {
      await logout();
      return false;
    }
  }

  Future<void> _setSession(AuthSession session) async {
    _session = session;
    await _storage.write(session);
    notifyListeners();
  }

  void _setBusy(bool value) {
    _isBusy = value;
    notifyListeners();
  }
}
