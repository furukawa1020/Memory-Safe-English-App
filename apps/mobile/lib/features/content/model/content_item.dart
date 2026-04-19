class ContentItem {
  const ContentItem({
    required this.contentId,
    required this.title,
    required this.contentType,
    required this.level,
    required this.topic,
    required this.language,
    required this.rawText,
    required this.summaryText,
  });

  final String contentId;
  final String title;
  final String contentType;
  final String level;
  final String topic;
  final String language;
  final String rawText;
  final String summaryText;

  factory ContentItem.fromJson(Map<String, dynamic> json) {
    return ContentItem(
      contentId: json['content_id'] as String? ?? '',
      title: json['title'] as String? ?? '',
      contentType: json['content_type'] as String? ?? '',
      level: json['level'] as String? ?? '',
      topic: json['topic'] as String? ?? '',
      language: json['language'] as String? ?? '',
      rawText: json['raw_text'] as String? ?? '',
      summaryText: json['summary_text'] as String? ?? '',
    );
  }
}
