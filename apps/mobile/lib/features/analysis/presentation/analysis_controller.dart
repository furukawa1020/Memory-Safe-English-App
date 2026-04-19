import 'package:flutter/foundation.dart';

import '../../content/data/content_repository.dart';
import '../../content/model/chunking_result.dart';

class AnalysisController extends ChangeNotifier {
  AnalysisController(this._repository);

  final ContentRepository _repository;

  bool isSubmitting = false;
  String? errorText;
  ChunkingResult? result;

  Future<void> analyze(String text) async {
    if (text.trim().isEmpty) {
      errorText = 'Enter some English text first.';
      notifyListeners();
      return;
    }

    isSubmitting = true;
    errorText = null;
    notifyListeners();

    try {
      result = await _repository.analyzeText(text);
    } catch (_) {
      errorText = 'Analysis failed. Please try again.';
    } finally {
      isSubmitting = false;
      notifyListeners();
    }
  }
}
