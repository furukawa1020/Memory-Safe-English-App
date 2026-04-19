import 'package:flutter/foundation.dart';

import '../features/auth/data/auth_repository.dart';
import '../features/auth/model/auth_session.dart';

class SessionController extends ChangeNotifier {
  AuthSession? _session;
  bool _isBusy = false;

  AuthSession? get session => _session;
  String? get accessToken => _session?.accessToken;
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

  void _setBusy(bool value) {
    _isBusy = value;
    notifyListeners();
  }
}
