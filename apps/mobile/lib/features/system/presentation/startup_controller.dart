import 'package:flutter/foundation.dart';

import '../data/system_repository.dart';
import '../model/backend_status.dart';

class StartupController extends ChangeNotifier {
  StartupController(this._repository);

  final SystemRepository _repository;

  bool isLoading = false;
  String? errorText;
  BackendStatus? backendStatus;

  bool get isReady => backendStatus?.ready == true;

  Future<void> load() async {
    isLoading = true;
    errorText = null;
    notifyListeners();

    try {
      backendStatus = await _repository.fetchBackendStatus();
    } catch (_) {
      errorText = 'Could not reach the backend yet.';
    } finally {
      isLoading = false;
      notifyListeners();
    }
  }
}
