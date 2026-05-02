import '../../../core/api/api_client.dart';
import '../model/chunking_result.dart';
import '../model/content_item.dart';
import '../model/problem_item.dart';
import '../model/rust_problem_dashboard.dart';

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

  Future<AdaptiveSessionResultItem> fetchAdaptiveSession(
    String text, {
    String targetContext = 'meeting',
    List<String> selfReportedDifficulties = const [
      'audio_tracking',
      'speech_breakdown',
    ],
    String fatigueLevel = 'medium',
    List<Map<String, dynamic>> sessionEvents = const [
      <String, dynamic>{
        'event_type': 'audio_pause',
        'chunk_order': 1,
        'seconds': 1.2,
      },
      <String, dynamic>{
        'event_type': 'repeat',
        'chunk_order': 2,
        'seconds': 0.0,
      },
    ],
  }) async {
    final response = await _apiClient.post(
      '/analysis/adaptive-session',
      body: <String, dynamic>{
        'text': text,
        'language': 'en',
        'target_context': targetContext,
        'self_reported_difficulties': selfReportedDifficulties,
        'fatigue_level': fatigueLevel,
        'session_events': sessionEvents,
      },
    );
    return AdaptiveSessionResultItem.fromJson(response);
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

  Future<List<ProblemItem>> fetchCustomProblems({
    String? source,
    bool pinnedOnly = false,
    int limit = 30,
  }) async {
    final query = <String>[
      if (source != null && source.isNotEmpty)
        'source=${Uri.encodeQueryComponent(source)}',
      'pinned_only=$pinnedOnly',
      'limit=$limit',
    ].join('&');
    final response = await _apiClient.get('/problem-bank/custom?$query');
    final items = response['items'] as List<dynamic>? ?? const [];
    return items
        .map((item) => ProblemItem.fromJson(item as Map<String, dynamic>))
        .toList(growable: false);
  }

  Future<ProblemItem> saveProblem(String problemId) async {
    final response = await _apiClient.post(
      '/problem-bank/$problemId/save',
      body: const <String, dynamic>{'source': 'reviewed'},
    );
    final items = response['items'] as List<dynamic>? ?? const [];
    if (items.isEmpty) {
      throw StateError('Saved problem response did not include an item.');
    }
    return ProblemItem.fromJson(items.first as Map<String, dynamic>);
  }

  Future<ProblemItem> pinProblem(String problemId, {required bool pinned}) async {
    final response = await _apiClient.patch(
      '/problem-bank/$problemId',
      body: <String, dynamic>{'pinned': pinned},
    );
    return ProblemItem.fromJson(response);
  }

  Future<ProblemItem> addProblemNote(
    String problemId, {
    required String notes,
  }) async {
    final response = await _apiClient.patch(
      '/problem-bank/$problemId',
      body: <String, dynamic>{'notes': notes},
    );
    return ProblemItem.fromJson(response);
  }

  Future<ProblemItem> recordProblemUsage(
    String problemId, {
    required bool successful,
    String? note,
  }) async {
    final response = await _apiClient.post(
      '/problem-bank/$problemId/usage',
      body: <String, dynamic>{
        'successful': successful,
        if (note != null && note.isNotEmpty) 'append_note': note,
      },
    );
    return ProblemItem.fromJson(response);
  }

  Future<RustProblemDashboard> fetchProblemDashboard() async {
    final response = await _apiClient.get(
      '/problem-bank/dashboard?preferred_mode=speaking&activity_source=reviewed&stale_source=reviewed',
    );
    return RustProblemDashboard.fromJson(response);
  }

  Future<List<RustProblemSnapshot>> fetchProblemSnapshots({int limit = 5}) async {
    final response = await _apiClient.get('/problem-bank/snapshots?limit=$limit');
    final items = response['items'] as List<dynamic>? ?? const [];
    return items
        .map((item) => RustProblemSnapshot.fromJson(item as Map<String, dynamic>))
        .toList(growable: false);
  }

  Future<RustProblemSnapshot> captureProblemSnapshot({String? note}) async {
    final response = await _apiClient.post(
      '/problem-bank/snapshots/capture?preferred_mode=speaking&activity_source=reviewed&stale_source=reviewed',
      body: <String, dynamic>{
        if (note != null && note.isNotEmpty) 'note': note,
      },
    );
    return RustProblemSnapshot.fromJson(response);
  }
}
