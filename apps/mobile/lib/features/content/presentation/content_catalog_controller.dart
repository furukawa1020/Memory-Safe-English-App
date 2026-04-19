import 'package:flutter/foundation.dart';

import '../data/content_repository.dart';
import '../model/content_item.dart';

class ContentCatalogController extends ChangeNotifier {
  ContentCatalogController(this._repository);

  final ContentRepository _repository;

  List<ContentItem> items = const [];
  bool isLoading = false;
  String? errorText;

  Future<void> load() async {
    isLoading = true;
    errorText = null;
    notifyListeners();

    try {
      items = await _repository.fetchContents();
    } catch (_) {
      errorText = 'Could not load content yet. Please try again.';
    } finally {
      isLoading = false;
      notifyListeners();
    }
  }
}
