class ChunkItem {
  const ChunkItem({
    required this.order,
    required this.text,
    required this.role,
    required this.skeletonRank,
  });

  final int order;
  final String text;
  final String role;
  final int skeletonRank;

  bool get isCore => skeletonRank == 1 || role == 'core';

  factory ChunkItem.fromJson(Map<String, dynamic> json) {
    return ChunkItem(
      order: json['order'] as int? ?? 0,
      text: json['text'] as String? ?? '',
      role: json['role'] as String? ?? '',
      skeletonRank: json['skeleton_rank'] as int? ?? 2,
    );
  }
}

class ChunkingResult {
  const ChunkingResult({
    required this.version,
    required this.language,
    required this.summary,
    required this.chunks,
  });

  final String version;
  final String language;
  final String summary;
  final List<ChunkItem> chunks;

  factory ChunkingResult.fromJson(Map<String, dynamic> json) {
    final chunkList = json['chunks'] as List<dynamic>? ?? const [];
    return ChunkingResult(
      version: json['version'] as String? ?? '',
      language: json['language'] as String? ?? 'en',
      summary: json['summary'] as String? ?? '',
      chunks: chunkList.map((item) => ChunkItem.fromJson(item as Map<String, dynamic>)).toList(),
    );
  }
}

class SkeletonPart {
  const SkeletonPart({
    required this.order,
    required this.text,
    required this.role,
    required this.emphasis,
  });

  final int order;
  final String text;
  final String role;
  final int emphasis;

  bool get isCore => role == 'core' || emphasis >= 2;

  factory SkeletonPart.fromJson(Map<String, dynamic> json) {
    return SkeletonPart(
      order: json['order'] as int? ?? 0,
      text: json['text'] as String? ?? '',
      role: json['role'] as String? ?? '',
      emphasis: json['emphasis'] as int? ?? 1,
    );
  }
}

class SkeletonResult {
  const SkeletonResult({
    required this.version,
    required this.language,
    required this.summary,
    required this.parts,
  });

  final String version;
  final String language;
  final String summary;
  final List<SkeletonPart> parts;

  factory SkeletonResult.fromJson(Map<String, dynamic> json) {
    final partList = json['parts'] as List<dynamic>? ?? const [];
    return SkeletonResult(
      version: json['version'] as String? ?? '',
      language: json['language'] as String? ?? 'en',
      summary: json['summary'] as String? ?? '',
      parts: partList.map((item) => SkeletonPart.fromJson(item as Map<String, dynamic>)).toList(),
    );
  }
}

class ReaderFocusStep {
  const ReaderFocusStep({
    required this.step,
    required this.chunkOrder,
    required this.text,
    required this.role,
    required this.supportBefore,
    required this.supportAfter,
    required this.supportDensity,
    required this.overloadRisk,
    required this.presentationHint,
    required this.guidanceJa,
    required this.guidanceEn,
  });

  final int step;
  final int chunkOrder;
  final String text;
  final String role;
  final List<String> supportBefore;
  final List<String> supportAfter;
  final String supportDensity;
  final String overloadRisk;
  final String presentationHint;
  final String guidanceJa;
  final String guidanceEn;

  factory ReaderFocusStep.fromJson(Map<String, dynamic> json) {
    List<String> readList(String key) {
      final values = json[key] as List<dynamic>? ?? const [];
      return values.map((item) => item.toString()).toList();
    }

    return ReaderFocusStep(
      step: json['step'] as int? ?? 0,
      chunkOrder: json['chunk_order'] as int? ?? 0,
      text: json['text'] as String? ?? '',
      role: json['role'] as String? ?? '',
      supportBefore: readList('support_before'),
      supportAfter: readList('support_after'),
      supportDensity: json['support_density'] as String? ?? '',
      overloadRisk: json['overload_risk'] as String? ?? '',
      presentationHint: json['presentation_hint'] as String? ?? '',
      guidanceJa: json['guidance_ja'] as String? ?? '',
      guidanceEn: json['guidance_en'] as String? ?? '',
    );
  }
}

class ReaderHotspot {
  const ReaderHotspot({
    required this.chunkOrder,
    required this.text,
    required this.riskLevel,
    required this.reason,
    required this.recommendation,
  });

  final int chunkOrder;
  final String text;
  final String riskLevel;
  final String reason;
  final String recommendation;

  factory ReaderHotspot.fromJson(Map<String, dynamic> json) {
    return ReaderHotspot(
      chunkOrder: json['chunk_order'] as int? ?? 0,
      text: json['text'] as String? ?? '',
      riskLevel: json['risk_level'] as String? ?? '',
      reason: json['reason'] as String? ?? '',
      recommendation: json['recommendation'] as String? ?? '',
    );
  }
}

class ReaderPlanResult {
  const ReaderPlanResult({
    required this.version,
    required this.language,
    required this.summary,
    required this.recommendedMode,
    required this.displayStrategy,
    required this.focusSteps,
    required this.hotspots,
  });

  final String version;
  final String language;
  final String summary;
  final String recommendedMode;
  final String displayStrategy;
  final List<ReaderFocusStep> focusSteps;
  final List<ReaderHotspot> hotspots;

  factory ReaderPlanResult.fromJson(Map<String, dynamic> json) {
    final focusStepList = json['focus_steps'] as List<dynamic>? ?? const [];
    final hotspotList = json['hotspots'] as List<dynamic>? ?? const [];

    return ReaderPlanResult(
      version: json['version'] as String? ?? '',
      language: json['language'] as String? ?? 'en',
      summary: json['summary'] as String? ?? '',
      recommendedMode: json['recommended_mode'] as String? ?? '',
      displayStrategy: json['display_strategy'] as String? ?? '',
      focusSteps: focusStepList
          .map((item) => ReaderFocusStep.fromJson(item as Map<String, dynamic>))
          .toList(),
      hotspots: hotspotList
          .map((item) => ReaderHotspot.fromJson(item as Map<String, dynamic>))
          .toList(),
    );
  }
}

class ListeningPausePoint {
  const ListeningPausePoint({
    required this.index,
    required this.afterChunkOrder,
    required this.pauseReason,
    required this.cueEn,
    required this.cueJa,
    required this.previewText,
    required this.riskLevel,
  });

  final int index;
  final int afterChunkOrder;
  final String pauseReason;
  final String cueEn;
  final String cueJa;
  final String previewText;
  final String riskLevel;

  factory ListeningPausePoint.fromJson(Map<String, dynamic> json) {
    return ListeningPausePoint(
      index: json['index'] as int? ?? 0,
      afterChunkOrder: json['after_chunk_order'] as int? ?? 0,
      pauseReason: json['pause_reason'] as String? ?? '',
      cueEn: json['cue_en'] as String? ?? '',
      cueJa: json['cue_ja'] as String? ?? '',
      previewText: json['preview_text'] as String? ?? '',
      riskLevel: json['risk_level'] as String? ?? '',
    );
  }
}

class ListeningPlanResult {
  const ListeningPlanResult({
    required this.version,
    required this.language,
    required this.summary,
    required this.recommendedSpeed,
    required this.pausePoints,
    required this.finalPassStrategy,
  });

  final String version;
  final String language;
  final String summary;
  final String recommendedSpeed;
  final List<ListeningPausePoint> pausePoints;
  final String finalPassStrategy;

  factory ListeningPlanResult.fromJson(Map<String, dynamic> json) {
    final points = json['pause_points'] as List<dynamic>? ?? const [];
    return ListeningPlanResult(
      version: json['version'] as String? ?? '',
      language: json['language'] as String? ?? 'en',
      summary: json['summary'] as String? ?? '',
      recommendedSpeed: json['recommended_speed'] as String? ?? '',
      pausePoints: points
          .map((item) => ListeningPausePoint.fromJson(item as Map<String, dynamic>))
          .toList(),
      finalPassStrategy: json['final_pass_strategy'] as String? ?? '',
    );
  }
}

class SpeakingStepItem {
  const SpeakingStepItem({
    required this.step,
    required this.text,
    required this.purpose,
    required this.riskLevel,
    required this.deliveryTipJa,
    required this.deliveryTipEn,
  });

  final int step;
  final String text;
  final String purpose;
  final String riskLevel;
  final String deliveryTipJa;
  final String deliveryTipEn;

  factory SpeakingStepItem.fromJson(Map<String, dynamic> json) {
    return SpeakingStepItem(
      step: json['step'] as int? ?? 0,
      text: json['text'] as String? ?? '',
      purpose: json['purpose'] as String? ?? '',
      riskLevel: json['risk_level'] as String? ?? '',
      deliveryTipJa: json['delivery_tip_ja'] as String? ?? '',
      deliveryTipEn: json['delivery_tip_en'] as String? ?? '',
    );
  }
}

class SpeakingPlanResult {
  const SpeakingPlanResult({
    required this.version,
    required this.language,
    required this.summary,
    required this.recommendedStyle,
    required this.openerOptions,
    required this.bridgePhrases,
    required this.steps,
    required this.rescuePhrases,
  });

  final String version;
  final String language;
  final String summary;
  final String recommendedStyle;
  final List<String> openerOptions;
  final List<String> bridgePhrases;
  final List<SpeakingStepItem> steps;
  final List<String> rescuePhrases;

  factory SpeakingPlanResult.fromJson(Map<String, dynamic> json) {
    List<String> readStringList(String key) {
      final values = json[key] as List<dynamic>? ?? const [];
      return values.map((item) => item.toString()).toList();
    }

    final steps = json['steps'] as List<dynamic>? ?? const [];
    return SpeakingPlanResult(
      version: json['version'] as String? ?? '',
      language: json['language'] as String? ?? 'en',
      summary: json['summary'] as String? ?? '',
      recommendedStyle: json['recommended_style'] as String? ?? '',
      openerOptions: readStringList('opener_options'),
      bridgePhrases: readStringList('bridge_phrases'),
      rescuePhrases: readStringList('rescue_phrases'),
      steps: steps
          .map((item) => SpeakingStepItem.fromJson(item as Map<String, dynamic>))
          .toList(),
    );
  }
}
