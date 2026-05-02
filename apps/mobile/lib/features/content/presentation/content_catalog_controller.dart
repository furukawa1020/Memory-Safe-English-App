import 'package:flutter/foundation.dart';

import '../data/content_repository.dart';
import '../model/content_item.dart';
import '../model/problem_item.dart';
import '../model/rust_problem_dashboard.dart';
import '../model/chunking_result.dart';

class ContentCatalogController extends ChangeNotifier {
  ContentCatalogController(this._repository);

  final ContentRepository _repository;

  List<ContentItem> items = const [];
  List<ProblemItem> recommendedItems = const [];
  RustProblemDashboard? rustDashboard;
  List<RustProblemSnapshot> rustSnapshots = const [];
  AdaptiveSessionResultItem? adaptiveSession;
  bool isLoading = false;
  bool isCapturingSnapshot = false;
  String? rustDashboardErrorText;
  String? problemActionErrorText;
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
      final adaptiveSessionFuture = _repository.fetchAdaptiveSession(
        'The client approved the design draft, but the delivery schedule is still under review.',
      );
      final dashboardFuture = _repository.fetchProblemDashboard();
      final snapshotsFuture = _repository.fetchProblemSnapshots(limit: 5);
      items = await contentsFuture;
      recommendedItems = await recommendedFuture;
      adaptiveSession = await adaptiveSessionFuture;
      rustDashboard = await dashboardFuture;
      rustSnapshots = await snapshotsFuture;
      problemActionErrorText = null;
      rustDashboardErrorText = null;
    } catch (_) {
      errorText = 'Could not load content yet. Please try again.';
      rustDashboardErrorText = 'Could not load the Rust dashboard yet.';
    } finally {
      isLoading = false;
      notifyListeners();
    }
  }

  Future<void> saveProblem(ProblemItem item) async {
    try {
      final saved = await _repository.saveProblem(item.id);
      _replaceProblem(item.id, saved);
      problemActionErrorText = null;
    } catch (_) {
      problemActionErrorText = 'Could not save the problem yet.';
    } finally {
      notifyListeners();
    }
  }

  Future<void> togglePinned(ProblemItem item) async {
    try {
      final updated = await _repository.pinProblem(item.id, pinned: !item.pinned);
      _replaceProblem(item.id, updated);
      problemActionErrorText = null;
    } catch (_) {
      problemActionErrorText = 'Could not update the pin yet.';
    } finally {
      notifyListeners();
    }
  }

  Future<void> recordProblemUsage(
    ProblemItem item, {
    required bool successful,
    String? note,
  }) async {
    try {
      final updated = await _repository.recordProblemUsage(
        item.id,
        successful: successful,
        note: note,
      );
      _replaceProblem(item.id, updated);
      problemActionErrorText = null;
    } catch (_) {
      problemActionErrorText = 'Could not record progress yet.';
    } finally {
      notifyListeners();
    }
  }

  Future<void> updateProblemNotes(ProblemItem item, String notes) async {
    try {
      final updated = await _repository.addProblemNote(item.id, notes: notes);
      _replaceProblem(item.id, updated);
      problemActionErrorText = null;
    } catch (_) {
      problemActionErrorText = 'Could not save notes yet.';
    } finally {
      notifyListeners();
    }
  }

  Future<void> captureSnapshot({String? note}) async {
    isCapturingSnapshot = true;
    notifyListeners();
    try {
      await _repository.captureProblemSnapshot(note: note);
      rustDashboard = await _repository.fetchProblemDashboard();
      rustSnapshots = await _repository.fetchProblemSnapshots(limit: 5);
      rustDashboardErrorText = null;
    } catch (_) {
      rustDashboardErrorText = 'Could not capture the Rust snapshot yet.';
    } finally {
      isCapturingSnapshot = false;
      notifyListeners();
    }
  }

  void _replaceProblem(String oldId, ProblemItem updated) {
    recommendedItems = recommendedItems
        .map((item) => item.id == oldId ? updated : item)
        .toList(growable: false);
  }
}
