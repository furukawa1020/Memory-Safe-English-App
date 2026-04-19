import 'package:flutter/material.dart';

import '../config/app_config.dart';
import '../core/api/api_client.dart';
import '../features/auth/data/auth_repository.dart';
import '../features/auth/presentation/auth_screen.dart';
import '../features/content/data/content_repository.dart';
import '../features/content/presentation/home_screen.dart';
import '../features/system/data/system_repository.dart';
import '../features/system/presentation/startup_controller.dart';
import '../features/system/presentation/startup_gate.dart';
import 'app_scope.dart';
import 'session_controller.dart';
import 'theme.dart';

class MemorySafeEnglishApp extends StatelessWidget {
  const MemorySafeEnglishApp({
    super.key,
    required this.sessionController,
  });

  final SessionController sessionController;

  static Widget bootstrap() {
    final config = AppConfig.fromEnvironment();
    final sessionController = SessionController();
    final apiClient = ApiClient(baseUrl: config.apiBaseUrl, sessionController: sessionController);
    final authRepository = AuthRepository(apiClient);
    final contentRepository = ContentRepository(apiClient);
    final systemRepository = SystemRepository(apiClient);
    final startupController = StartupController(systemRepository);

    return AppScope(
      config: config,
      sessionController: sessionController,
      apiClient: apiClient,
      authRepository: authRepository,
      contentRepository: contentRepository,
      systemRepository: systemRepository,
      startupController: startupController,
      child: MemorySafeEnglishApp(sessionController: sessionController),
    );
  }

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    return AnimatedBuilder(
      animation: sessionController,
      builder: (context, _) {
        final startupGate = StartupGate(
          controller: scope.startupController,
          readyChild: scope.sessionController.isAuthenticated
              ? const HomeScreen()
              : const AuthScreen(),
        );

        return MaterialApp(
          title: 'Memory-Safe English',
          debugShowCheckedModeBanner: false,
          theme: buildAppTheme(),
          home: startupGate,
        );
      },
    );
  }
}
