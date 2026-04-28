import '../../../core/api/api_client.dart';
import '../model/chunking_result.dart';
import '../model/content_item.dart';
import '../model/problem_item.dart';

class ContentRepository {
  ContentRepository(this._apiClient);

  final ApiClient _apiClient;

  Future<List<ContentItem>> fetchContents() async {
    final response = await _apiClient.get('/contents');
    final items = response['items'] as List<dynamic>? ?? const [];
    return items.map((item) => ContentItem.fromJson(item as Map<String, dynamic>)).toList();
  }

  Future<ContentItem> fetchContent(String contentId) async {
    final response = await _apiClient.get('/contents/$contentId');
    return ContentItem.fromJson(response);
  }

  Future<ChunkingResult> fetchChunks(String contentId) async {
    final response = await _apiClient.get('/contents/$contentId/chunks');
    return ChunkingResult.fromJson(response);
  }

  Future<SkeletonResult> fetchSkeleton(String contentId) async {
    final response = await _apiClient.get('/contents/$contentId/skeleton');
    return SkeletonResult.fromJson(response);
  }

  Future<ReaderPlanResult> fetchReaderPlan(String text) async {
    final response = await _apiClient.post(
      '/analysis/reader-plan',
      body: <String, dynamic>{'text': text, 'language': 'en'},
    );
    return ReaderPlanResult.fromJson(response);
  }

  Future<ListeningPlanResult> fetchListeningPlan(String text) async {
    final response = await _apiClient.post(
      '/analysis/listening-plan',
      body: <String, dynamic>{'text': text, 'language': 'en'},
    );
    return ListeningPlanResult.fromJson(response);
  }

  Future<SpeakingPlanResult> fetchSpeakingPlan(String text) async {
    final response = await _apiClient.post(
      '/analysis/speaking-plan',
      body: <String, dynamic>{'text': text, 'language': 'en'},
    );
    return SpeakingPlanResult.fromJson(response);
  }

  Future<ChunkingResult> analyzeText(String text) async {
    final response = await _apiClient.post(
      '/analysis/chunks',
      body: <String, dynamic>{'text': text, 'language': 'en'},
    );
    return ChunkingResult.fromJson(response);
  }

  Future<List<ProblemItem>> fetchRecommendedProblems({
    String? preferredMode,
    String? targetContext,
    String? levelBand,
    String? topic,
    String? focusTag,
    int limit = 4,
  }) async {
    final query = <String>[
      if (preferredMode != null && preferredMode.isNotEmpty)
        'preferred_mode=${Uri.encodeQueryComponent(preferredMode)}',
      if (targetContext != null && targetContext.isNotEmpty)
        'target_context=${Uri.encodeQueryComponent(targetContext)}',
      if (levelBand != null && levelBand.isNotEmpty)
        'level_band=${Uri.encodeQueryComponent(levelBand)}',
      if (topic != null && topic.isNotEmpty)
        'topic=${Uri.encodeQueryComponent(topic)}',
      if (focusTag != null && focusTag.isNotEmpty)
        'focus_tag=${Uri.encodeQueryComponent(focusTag)}',
      'limit=$limit',
    ].join('&');
    final response = await _apiClient.get('/problem-bank/recommend?$query');
    final items = response['items'] as List<dynamic>? ?? const [];
    return items
        .map((item) => ProblemItem.fromJson(item as Map<String, dynamic>))
        .toList();
  }
}
