import 'package:flutter/foundation.dart';

import '../../../app/session_controller.dart';
import '../../auth/data/auth_repository.dart';
import '../data/system_repository.dart';
import '../model/backend_status.dart';
import '../model/mobile_bootstrap.dart';

class StartupController extends ChangeNotifier {
  StartupController(this._repository, this._sessionController, this._authRepository);

  final SystemRepository _repository;
  final SessionController _sessionController;
  final AuthRepository _authRepository;

  bool isLoading = false;
  String? errorText;
  BackendStatus? backendStatus;
  MobileBootstrap? bootstrap;
  bool restoredSession = false;

  bool get isReady => backendStatus?.ready == true;

  Future<void> load() async {
    isLoading = true;
    errorText = null;
    notifyListeners();

    try {
      await _sessionController.restore();
      restoredSession = _sessionController.isAuthenticated;
      bootstrap = await _repository.fetchMobileBootstrap();
      backendStatus = bootstrap!.backendStatus;
      if (_sessionController.isAuthenticated) {
        final refreshed = await _sessionController.refreshWith((refreshToken) async {
          final session = await _authRepository.refreshSession(refreshToken);
          return <String, dynamic>{
            'user': <String, dynamic>{
              'user_id': session.userId,
              'email': session.email,
              'display_name': session.displayName,
            },
            'tokens': <String, dynamic>{
              'access_token': session.accessToken,
              'refresh_token': session.refreshToken,
            },
          };
        });
        restoredSession = refreshed;
      }
    } catch (_) {
      errorText = 'Could not reach the backend yet.';
    } finally {
      isLoading = false;
      notifyListeners();
    }
  }
}
