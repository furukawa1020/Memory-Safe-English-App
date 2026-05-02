import 'backend_status.dart';

class MobileBootstrap {
  const MobileBootstrap({
    required this.ready,
    required this.checkedAtUnixMs,
    required this.recommendedBaseUrls,
    required this.routes,
    required this.capabilities,
    required this.api,
    required this.worker,
  });

  final bool ready;
  final int checkedAtUnixMs;
  final RecommendedBaseUrls recommendedBaseUrls;
  final FrontendRoutes routes;
  final FrontendCapabilities capabilities;
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
      capabilities: FrontendCapabilities.fromJson(
        json['capabilities'] as Map<String, dynamic>? ??
            const <String, dynamic>{},
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
    required this.refresh,
    required this.currentUser,
    required this.contents,
    required this.chunkAnalysis,
    required this.skeletonAnalysis,
    required this.readerPlan,
    required this.listeningPlan,
    required this.speakingPlan,
    required this.rescuePlan,
    required this.adaptiveSession,
  });

  final String readiness;
  final String login;
  final String register;
  final String refresh;
  final String currentUser;
  final String contents;
  final String chunkAnalysis;
  final String skeletonAnalysis;
  final String readerPlan;
  final String listeningPlan;
  final String speakingPlan;
  final String rescuePlan;
  final String adaptiveSession;

  factory FrontendRoutes.fromJson(Map<String, dynamic> json) {
    return FrontendRoutes(
      readiness: json['readiness'] as String? ?? '',
      login: json['login'] as String? ?? '',
      register: json['register'] as String? ?? '',
      refresh: json['refresh'] as String? ?? '',
      currentUser: json['current_user'] as String? ?? '',
      contents: json['contents'] as String? ?? '',
      chunkAnalysis: json['chunk_analysis'] as String? ?? '',
      skeletonAnalysis: json['skeleton_analysis'] as String? ?? '',
      readerPlan: json['reader_plan'] as String? ?? '',
      listeningPlan: json['listening_plan'] as String? ?? '',
      speakingPlan: json['speaking_plan'] as String? ?? '',
      rescuePlan: json['rescue_plan'] as String? ?? '',
      adaptiveSession: json['adaptive_session'] as String? ?? '',
    );
  }
}

class FrontendCapabilities {
  const FrontendCapabilities({
    required this.chunkReader,
    required this.skeletonReader,
    required this.readerPlan,
    required this.listeningPlan,
    required this.speakingPlan,
    required this.rescuePlan,
    required this.onboardingAssessment,
    required this.analyticsSummary,
    required this.adaptiveSession,
  });

  final bool chunkReader;
  final bool skeletonReader;
  final bool readerPlan;
  final bool listeningPlan;
  final bool speakingPlan;
  final bool rescuePlan;
  final bool onboardingAssessment;
  final bool analyticsSummary;
  final bool adaptiveSession;

  factory FrontendCapabilities.fromJson(Map<String, dynamic> json) {
    bool readBool(String key) => json[key] as bool? ?? false;

    return FrontendCapabilities(
      chunkReader: readBool('chunk_reader'),
      skeletonReader: readBool('skeleton_reader'),
      readerPlan: readBool('reader_plan'),
      listeningPlan: readBool('listening_plan'),
      speakingPlan: readBool('speaking_plan'),
      rescuePlan: readBool('rescue_plan'),
      onboardingAssessment: readBool('onboarding_assessment'),
      analyticsSummary: readBool('analytics_summary'),
      adaptiveSession: readBool('adaptive_session'),
    );
  }
}
