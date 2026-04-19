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
