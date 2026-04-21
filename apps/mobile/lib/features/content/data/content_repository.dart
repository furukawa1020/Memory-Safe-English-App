import '../../../core/api/api_client.dart';
import '../model/chunking_result.dart';
import '../model/content_item.dart';

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

  Future<ChunkingResult> analyzeText(String text) async {
    final response = await _apiClient.post(
      '/analysis/chunks',
      body: <String, dynamic>{'text': text, 'language': 'en'},
    );
    return ChunkingResult.fromJson(response);
  }
}
