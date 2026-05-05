import 'package:flutter/foundation.dart';

import '../../content/data/content_repository.dart';
import '../../content/model/chunking_result.dart';

enum AnalysisMode { chunks, listening, speaking, adaptive }

class AnalysisController extends ChangeNotifier {
  AnalysisController(this._repository);

  final ContentRepository _repository;
  String inputText =
      'In this study, we propose a memory safe interface that reduces cognitive overload during English reading.';
  int inputRevision = 0;

  bool isSubmitting = false;
  String? errorText;
  AnalysisMode mode = AnalysisMode.chunks;
  ChunkingResult? chunkResult;
  ListeningPlanResult? listeningResult;
  SpeakingPlanResult? speakingResult;
  AdaptiveSessionResultItem? adaptiveSessionResult;

  void setMode(AnalysisMode value) {
    mode = value;
    errorText = null;
    notifyListeners();
  }

  void setInputText(String value) {
    if (inputText == value) {
      return;
    }
    inputText = value;
    inputRevision++;
    notifyListeners();
  }

  Future<void> openPreset({
    required AnalysisMode nextMode,
    required String text,
    bool analyzeNow = true,
  }) async {
    mode = nextMode;
    errorText = null;
    setInputText(text);
    if (analyzeNow) {
      await analyze();
    }
  }

  Future<void> analyze([String? text]) async {
    final draft = (text ?? inputText).trim();
    if (draft.isEmpty) {
      errorText = 'Enter some English text first.';
      notifyListeners();
      return;
    }

    if (text != null && text != inputText) {
      inputText = text;
      inputRevision++;
    }

    isSubmitting = true;
    errorText = null;
    notifyListeners();

    try {
      switch (mode) {
        case AnalysisMode.chunks:
          chunkResult = await _repository.analyzeText(draft);
          break;
        case AnalysisMode.listening:
          listeningResult = await _repository.fetchListeningPlan(draft);
          break;
        case AnalysisMode.speaking:
          speakingResult = await _repository.fetchSpeakingPlan(draft);
          break;
        case AnalysisMode.adaptive:
          adaptiveSessionResult = await _repository.fetchAdaptiveSession(draft);
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
