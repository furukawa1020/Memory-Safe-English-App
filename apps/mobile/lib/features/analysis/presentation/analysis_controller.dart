import 'package:flutter/foundation.dart';

import '../../content/data/content_repository.dart';
import '../../content/model/chunking_result.dart';

enum AnalysisMode { chunks, listening, speaking }

class AnalysisController extends ChangeNotifier {
  AnalysisController(this._repository);

  final ContentRepository _repository;

  bool isSubmitting = false;
  String? errorText;
  AnalysisMode mode = AnalysisMode.chunks;
  ChunkingResult? chunkResult;
  ListeningPlanResult? listeningResult;
  SpeakingPlanResult? speakingResult;

  void setMode(AnalysisMode value) {
    mode = value;
    errorText = null;
    notifyListeners();
  }

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
      switch (mode) {
        case AnalysisMode.chunks:
          chunkResult = await _repository.analyzeText(text);
          break;
        case AnalysisMode.listening:
          listeningResult = await _repository.fetchListeningPlan(text);
          break;
        case AnalysisMode.speaking:
          speakingResult = await _repository.fetchSpeakingPlan(text);
          break;
      }
    } catch (_) {
      errorText = 'Analysis failed. Please try again.';
    } finally {
      isSubmitting = false;
      notifyListeners();
    }
  }
}
