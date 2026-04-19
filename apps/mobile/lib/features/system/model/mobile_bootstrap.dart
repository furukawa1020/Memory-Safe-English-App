import 'backend_status.dart';

class MobileBootstrap {
  const MobileBootstrap({
    required this.ready,
    required this.checkedAtUnixMs,
    required this.recommendedBaseUrls,
    required this.routes,
    required this.api,
    required this.worker,
  });

  final bool ready;
  final int checkedAtUnixMs;
  final RecommendedBaseUrls recommendedBaseUrls;
  final FrontendRoutes routes;
  final UpstreamStatus api;
  final UpstreamStatus worker;

  BackendStatus get backendStatus => BackendStatus(
        ready: ready,
        checkedAtUnixMs: checkedAtUnixMs,
        api: api,
        worker: worker,
      );

  factory MobileBootstrap.fromJson(Map<String, dynamic> json) {
    return MobileBootstrap(
      ready: json['ready'] as bool? ?? false,
      checkedAtUnixMs: _readInt(json['checked_at_unix_ms']),
      recommendedBaseUrls: RecommendedBaseUrls.fromJson(
        json['recommended_base_urls'] as Map<String, dynamic>? ??
            const <String, dynamic>{},
      ),
      routes: FrontendRoutes.fromJson(
        json['routes'] as Map<String, dynamic>? ?? const <String, dynamic>{},
      ),
      api: UpstreamStatus.fromJson(
        json['api'] as Map<String, dynamic>? ?? const <String, dynamic>{},
        fallbackName: 'api',
      ),
      worker: UpstreamStatus.fromJson(
        json['worker'] as Map<String, dynamic>? ?? const <String, dynamic>{},
        fallbackName: 'worker',
      ),
    );
  }

  static int _readInt(Object? value) {
    if (value is int) {
      return value;
    }
    if (value is num) {
      return value.toInt();
    }
    return 0;
  }
}

class RecommendedBaseUrls {
  const RecommendedBaseUrls({
    required this.androidEmulator,
    required this.iosSimulator,
    required this.desktop,
  });

  final String androidEmulator;
  final String iosSimulator;
  final String desktop;

  factory RecommendedBaseUrls.fromJson(Map<String, dynamic> json) {
    return RecommendedBaseUrls(
      androidEmulator: json['android_emulator'] as String? ?? '',
      iosSimulator: json['ios_simulator'] as String? ?? '',
      desktop: json['desktop'] as String? ?? '',
    );
  }
}

class FrontendRoutes {
  const FrontendRoutes({
    required this.readiness,
    required this.login,
    required this.register,
    required this.currentUser,
    required this.contents,
    required this.chunkAnalysis,
  });

  final String readiness;
  final String login;
  final String register;
  final String currentUser;
  final String contents;
  final String chunkAnalysis;

  factory FrontendRoutes.fromJson(Map<String, dynamic> json) {
    return FrontendRoutes(
      readiness: json['readiness'] as String? ?? '',
      login: json['login'] as String? ?? '',
      register: json['register'] as String? ?? '',
      currentUser: json['current_user'] as String? ?? '',
      contents: json['contents'] as String? ?? '',
      chunkAnalysis: json['chunk_analysis'] as String? ?? '',
    );
  }
}
