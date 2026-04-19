import 'package:flutter/widgets.dart';

import '../config/app_config.dart';
import '../core/api/api_client.dart';
import '../features/auth/data/auth_repository.dart';
import '../features/content/data/content_repository.dart';
import '../features/system/data/system_repository.dart';
import '../features/system/presentation/startup_controller.dart';
import 'session_controller.dart';

class AppScope extends InheritedWidget {
  const AppScope({
    super.key,
    required this.config,
    required this.sessionController,
    required this.apiClient,
    required this.authRepository,
    required this.contentRepository,
    required this.systemRepository,
    required this.startupController,
    required super.child,
  });

  final AppConfig config;
  final SessionController sessionController;
  final ApiClient apiClient;
  final AuthRepository authRepository;
  final ContentRepository contentRepository;
  final SystemRepository systemRepository;
  final StartupController startupController;

  static AppScope of(BuildContext context) {
    final scope = context.dependOnInheritedWidgetOfExactType<AppScope>();
    assert(scope != null, 'AppScope not found in context');
    return scope!;
  }

  @override
  bool updateShouldNotify(AppScope oldWidget) {
    return config != oldWidget.config ||
        sessionController != oldWidget.sessionController ||
        apiClient != oldWidget.apiClient ||
        authRepository != oldWidget.authRepository ||
        contentRepository != oldWidget.contentRepository ||
        systemRepository != oldWidget.systemRepository ||
        startupController != oldWidget.startupController;
  }
}
