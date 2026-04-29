class RustProblemDashboard {
  const RustProblemDashboard({
    required this.totalProblems,
    required this.totalUsage,
    required this.customProblems,
    required this.riskLevel,
    required this.nextAction,
    required this.recommendedNextMode,
    required this.alerts,
    required this.modeSummary,
    required this.trend,
    required this.staleProblems,
  });

  final int totalProblems;
  final int totalUsage;
  final int customProblems;
  final String riskLevel;
  final String nextAction;
  final String? recommendedNextMode;
  final List<RustProblemAlert> alerts;
  final List<RustModeSummary> modeSummary;
  final RustTrend trend;
  final List<RustStaleProblem> staleProblems;

  factory RustProblemDashboard.fromJson(Map<String, dynamic> json) {
    final stats = json['stats'] as Map<String, dynamic>? ?? const {};
    final alerts = json['alerts'] as List<dynamic>? ?? const [];
    final modeSummary = json['mode_summary'] as List<dynamic>? ?? const [];
    final staleProblems = json['stale_problems'] as List<dynamic>? ?? const [];
    final trend = json['trend'] as Map<String, dynamic>? ?? const {};

    return RustProblemDashboard(
      totalProblems: stats['total'] as int? ?? 0,
      totalUsage: stats['total_usage'] as int? ?? 0,
      customProblems: stats['custom'] as int? ?? 0,
      riskLevel: json['risk_level'] as String? ?? 'low',
      nextAction: json['next_action'] as String? ?? '',
      recommendedNextMode: json['recommended_next_mode'] as String?,
      alerts: alerts
          .map((item) => RustProblemAlert.fromJson(item as Map<String, dynamic>))
          .toList(growable: false),
      modeSummary: modeSummary
          .map((item) => RustModeSummary.fromJson(item as Map<String, dynamic>))
          .toList(growable: false),
      trend: RustTrend.fromJson(trend),
      staleProblems: staleProblems
          .map((item) => RustStaleProblem.fromJson(item as Map<String, dynamic>))
          .toList(growable: false),
    );
  }
}

class RustProblemAlert {
  const RustProblemAlert({
    required this.level,
    required this.code,
    required this.message,
  });

  final String level;
  final String code;
  final String message;

  factory RustProblemAlert.fromJson(Map<String, dynamic> json) {
    return RustProblemAlert(
      level: json['level'] as String? ?? 'low',
      code: json['code'] as String? ?? '',
      message: json['message'] as String? ?? '',
    );
  }
}

class RustModeSummary {
  const RustModeSummary({
    required this.mode,
    required this.totalProblems,
    required this.totalUsage,
    required this.recentFailures,
    required this.staleCount,
    required this.successRate,
  });

  final String mode;
  final int totalProblems;
  final int totalUsage;
  final int recentFailures;
  final int staleCount;
  final double successRate;

  factory RustModeSummary.fromJson(Map<String, dynamic> json) {
    return RustModeSummary(
      mode: json['mode'] as String? ?? '',
      totalProblems: json['total_problems'] as int? ?? 0,
      totalUsage: json['total_usage'] as int? ?? 0,
      recentFailures: json['recent_failures'] as int? ?? 0,
      staleCount: json['stale_count'] as int? ?? 0,
      successRate: (json['success_rate'] as num?)?.toDouble() ?? 0,
    );
  }
}

class RustTrend {
  const RustTrend({
    required this.recentSuccessRate,
    required this.previousSuccessRate,
    required this.successRateDelta,
    required this.recentAttempts,
    required this.previousAttempts,
  });

  final double recentSuccessRate;
  final double previousSuccessRate;
  final double successRateDelta;
  final int recentAttempts;
  final int previousAttempts;

  factory RustTrend.fromJson(Map<String, dynamic> json) {
    return RustTrend(
      recentSuccessRate: (json['recent_success_rate'] as num?)?.toDouble() ?? 0,
      previousSuccessRate:
          (json['previous_success_rate'] as num?)?.toDouble() ?? 0,
      successRateDelta: (json['success_rate_delta'] as num?)?.toDouble() ?? 0,
      recentAttempts: json['recent_total_attempts'] as int? ?? 0,
      previousAttempts: json['previous_total_attempts'] as int? ?? 0,
    );
  }
}

class RustStaleProblem {
  const RustStaleProblem({
    required this.problemId,
    required this.title,
    required this.mode,
    required this.idleDays,
  });

  final String problemId;
  final String title;
  final String mode;
  final int idleDays;

  factory RustStaleProblem.fromJson(Map<String, dynamic> json) {
    return RustStaleProblem(
      problemId: json['problem_id'] as String? ?? '',
      title: json['title'] as String? ?? '',
      mode: json['mode'] as String? ?? '',
      idleDays: json['idle_days'] as int? ?? 0,
    );
  }
}

class RustProblemSnapshot {
  const RustProblemSnapshot({
    required this.id,
    required this.capturedAtUnix,
    required this.note,
    required this.riskLevel,
    required this.recommendedNextMode,
  });

  final String id;
  final int capturedAtUnix;
  final String note;
  final String riskLevel;
  final String? recommendedNextMode;

  factory RustProblemSnapshot.fromJson(Map<String, dynamic> json) {
    final dashboard = json['dashboard'] as Map<String, dynamic>? ?? const {};
    return RustProblemSnapshot(
      id: json['id'] as String? ?? '',
      capturedAtUnix: json['captured_at_unix'] as int? ?? 0,
      note: json['note'] as String? ?? '',
      riskLevel: dashboard['risk_level'] as String? ?? 'low',
      recommendedNextMode: dashboard['recommended_next_mode'] as String?,
    );
  }
}
