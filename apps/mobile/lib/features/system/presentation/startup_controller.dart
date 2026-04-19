import 'package:flutter/foundation.dart';

import '../../../app/session_controller.dart';
import '../data/system_repository.dart';
import '../model/backend_status.dart';
import '../model/mobile_bootstrap.dart';

class StartupController extends ChangeNotifier {
  StartupController(this._repository, this._sessionController);

  final SystemRepository _repository;
  final SessionController _sessionController;

  bool isLoading = false;
  String? errorText;
  BackendStatus? backendStatus;
  MobileBootstrap? bootstrap;

  bool get isReady => backendStatus?.ready == true;

  Future<void> load() async {
    isLoading = true;
    errorText = null;
    notifyListeners();

    try {
      await _sessionController.restore();
      bootstrap = await _repository.fetchMobileBootstrap();
      backendStatus = bootstrap!.backendStatus;
    } catch (_) {
      errorText = 'Could not reach the backend yet.';
    } finally {
      isLoading = false;
      notifyListeners();
    }
  }
}
