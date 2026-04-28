class ProblemItem {
  const ProblemItem({
    required this.id,
    required this.title,
    required this.mode,
    required this.levelBand,
    required this.topic,
    required this.targetContext,
    required this.prompt,
    required this.wmSupport,
    required this.successCheck,
    required this.tags,
    required this.source,
    required this.pinned,
    required this.usageCount,
    required this.successCount,
    required this.lastUsedUnix,
    required this.notes,
  });

  final String id;
  final String title;
  final String mode;
  final String levelBand;
  final String topic;
  final String targetContext;
  final String prompt;
  final String wmSupport;
  final String successCheck;
  final List<String> tags;
  final String source;
  final bool pinned;
  final int usageCount;
  final int successCount;
  final int lastUsedUnix;
  final String notes;

  bool get isSaved => id.startsWith('saved_') || tags.contains('saved');

  factory ProblemItem.fromJson(Map<String, dynamic> json) {
    final tags = json['tags'] as List<dynamic>? ?? const [];
    return ProblemItem(
      id: json['id'] as String? ?? '',
      title: json['title'] as String? ?? '',
      mode: json['mode'] as String? ?? '',
      levelBand: json['level_band'] as String? ?? '',
      topic: json['topic'] as String? ?? '',
      targetContext: json['target_context'] as String? ?? '',
      prompt: json['prompt'] as String? ?? '',
      wmSupport: json['wm_support'] as String? ?? '',
      successCheck: json['success_check'] as String? ?? '',
      tags: tags.map((item) => item.toString()).toList(),
      source: json['source'] as String? ?? '',
      pinned: json['pinned'] as bool? ?? false,
      usageCount: json['usage_count'] as int? ?? 0,
      successCount: json['success_count'] as int? ?? 0,
      lastUsedUnix: json['last_used_unix'] as int? ?? 0,
      notes: json['notes'] as String? ?? '',
    );
  }
}
