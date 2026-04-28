import 'package:flutter/foundation.dart';

import '../data/content_repository.dart';
import '../model/content_item.dart';
import '../model/problem_item.dart';

class ContentCatalogController extends ChangeNotifier {
  ContentCatalogController(this._repository);

  final ContentRepository _repository;

  List<ContentItem> items = const [];
  List<ProblemItem> recommendedItems = const [];
  bool isLoading = false;
  String? errorText;

  Future<void> load() async {
    isLoading = true;
    errorText = null;
    notifyListeners();

    try {
      final contentsFuture = _repository.fetchContents();
      final recommendedFuture = _repository.fetchRecommendedProblems(
        preferredMode: 'speaking',
        targetContext: 'meeting',
        levelBand: 'toeic_750_800',
        focusTag: 'status_update',
        limit: 4,
      );
      items = await contentsFuture;
      recommendedItems = await recommendedFuture;
    } catch (_) {
      errorText = 'Could not load content yet. Please try again.';
    } finally {
      isLoading = false;
      notifyListeners();
    }
  }
}
