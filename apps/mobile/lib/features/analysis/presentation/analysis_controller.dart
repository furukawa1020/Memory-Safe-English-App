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
      errorText = '英文を入力してください';
      notifyListeners();
      return;
    }

    isSubmitting = true;
    errorText = null;
    notifyListeners();
    try {
      result = await _repository.analyzeText(text);
    } catch (_) {
      errorText = '解析に失敗しました';
    } finally {
      isSubmitting = false;
      notifyListeners();
    }
  }
}
