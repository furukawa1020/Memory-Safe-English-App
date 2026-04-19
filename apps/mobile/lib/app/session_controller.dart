import 'package:flutter/foundation.dart';

import '../features/auth/data/auth_repository.dart';
import '../features/auth/model/auth_session.dart';

class SessionController extends ChangeNotifier {
  AuthSession? _session;
  bool _isBusy = false;
  Future<bool>? _refreshFuture;

  AuthSession? get session => _session;
  String? get accessToken => _session?.accessToken;
  String? get refreshToken => _session?.refreshToken;
  bool get isAuthenticated => _session != null;
  bool get isBusy => _isBusy;

  Future<void> login({
    required AuthRepository repository,
    required String email,
    required String password,
  }) async {
    _setBusy(true);
    try {
      _session = await repository.login(email: email, password: password);
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
      _session = await repository.register(
        email: email,
        password: password,
        displayName: displayName,
      );
    } finally {
      _setBusy(false);
    }
  }

  void logout() {
    _session = null;
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
      _session = AuthSession.fromJson(payload);
      notifyListeners();
      return _session?.accessToken.isNotEmpty == true;
    } catch (_) {
      logout();
      return false;
    }
  }

  void _setBusy(bool value) {
    _isBusy = value;
    notifyListeners();
  }
}
