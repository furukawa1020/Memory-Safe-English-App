import 'package:flutter/material.dart';

import '../../../app/app_scope.dart';
import '../data/content_repository.dart';
import '../model/problem_item.dart';
import '../model/rust_problem_dashboard.dart';

class ProblemBankScreen extends StatefulWidget {
  const ProblemBankScreen({
    super.key,
    this.initialTabIndex = 0,
  });

  final int initialTabIndex;

  @override
  State<ProblemBankScreen> createState() => _ProblemBankScreenState();
}

class _ProblemBankScreenState extends State<ProblemBankScreen> {
  ContentRepository? _repository;
  List<ProblemItem> _customProblems = const [];
  List<RustProblemSnapshot> _snapshots = const [];
  bool _isLoading = true;
  bool _isSaving = false;
  String? _errorText;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final repository = AppScope.of(context).contentRepository;
    if (_repository == repository) {
      return;
    }
    _repository = repository;
    _load();
  }

  Future<void> _load() async {
    final repository = _repository;
    if (repository == null) {
      return;
    }

    setState(() {
      _isLoading = true;
      _errorText = null;
    });

    try {
      final customProblems = await repository.fetchCustomProblems(
        source: 'reviewed',
        limit: 40,
      );
      final snapshots = await repository.fetchProblemSnapshots(limit: 20);
      if (!mounted) {
        return;
      }
      setState(() {
        _customProblems = customProblems;
        _snapshots = snapshots;
      });
    } catch (_) {
      if (!mounted) {
        return;
      }
      setState(() {
        _errorText = 'Rust problem bank could not be loaded yet.';
      });
    } finally {
      if (mounted) {
        setState(() {
          _isLoading = false;
        });
      }
    }
  }

  Future<void> _togglePinned(ProblemItem item) async {
    final repository = _repository;
    if (repository == null) {
      return;
    }

    setState(() {
      _isSaving = true;
      _errorText = null;
    });

    try {
      final updated = await repository.pinProblem(item.id, pinned: !item.pinned);
      _replaceProblem(updated);
    } catch (_) {
      _setError('Could not update the pin yet.');
    } finally {
      if (mounted) {
        setState(() {
          _isSaving = false;
        });
      }
    }
  }

  Future<void> _recordDone(ProblemItem item) async {
    final repository = _repository;
    if (repository == null) {
      return;
    }

    setState(() {
      _isSaving = true;
      _errorText = null;
    });

    try {
      final updated = await repository.recordProblemUsage(
        item.id,
        successful: true,
      );
      _replaceProblem(updated);
    } catch (_) {
      _setError('Could not record progress yet.');
    } finally {
      if (mounted) {
        setState(() {
          _isSaving = false;
        });
      }
    }
  }

  Future<void> _showProblemNoteDialog(ProblemItem item) async {
    final repository = _repository;
    if (repository == null) {
      return;
    }

    final controller = TextEditingController(text: item.notes);
    final nextValue = await showDialog<String>(
      context: context,
      builder: (context) {
        return AlertDialog(
          title: const Text('Problem note'),
          content: TextField(
            controller: controller,
            minLines: 3,
            maxLines: 5,
            decoration: const InputDecoration(
              hintText: 'Add a note about where this problem helped.',
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('Cancel'),
            ),
            FilledButton(
              onPressed: () => Navigator.of(context).pop(controller.text.trim()),
              child: const Text('Save note'),
            ),
          ],
        );
      },
    );
    controller.dispose();

    if (!mounted || nextValue == null) {
      return;
    }

    setState(() {
      _isSaving = true;
      _errorText = null;
    });

    try {
      final updated = await repository.addProblemNote(item.id, notes: nextValue);
      _replaceProblem(updated);
    } catch (_) {
      _setError('Could not save notes yet.');
    } finally {
      if (mounted) {
        setState(() {
          _isSaving = false;
        });
      }
    }
  }

  void _replaceProblem(ProblemItem updated) {
    if (!mounted) {
      return;
    }
    setState(() {
      _customProblems = _customProblems
          .map((item) => item.id == updated.id ? updated : item)
          .toList(growable: false);
    });
  }

  void _setError(String message) {
    if (!mounted) {
      return;
    }
    setState(() {
      _errorText = message;
    });
  }

  @override
  Widget build(BuildContext context) {
    return DefaultTabController(
      length: 2,
      initialIndex: widget.initialTabIndex.clamp(0, 1),
      child: Scaffold(
        appBar: AppBar(
          title: const Text('Rust Problem Bank'),
          bottom: const TabBar(
            tabs: [
              Tab(text: 'My Bank'),
              Tab(text: 'Snapshots'),
            ],
          ),
        ),
        body: SafeArea(
          child: Column(
            children: [
              if (_errorText != null)
                Padding(
                  padding: const EdgeInsets.fromLTRB(16, 12, 16, 0),
                  child: Text(
                    _errorText!,
                    style: TextStyle(
                      color: Theme.of(context).colorScheme.error,
                    ),
                  ),
                ),
              if (_isSaving)
                const Padding(
                  padding: EdgeInsets.fromLTRB(16, 12, 16, 0),
                  child: LinearProgressIndicator(),
                ),
              Expanded(
                child: TabBarView(
                  children: [
                    _buildProblemBankTab(),
                    _buildSnapshotTab(),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildProblemBankTab() {
    if (_isLoading) {
      return const Center(child: CircularProgressIndicator());
    }
    if (_customProblems.isEmpty) {
      return RefreshIndicator(
        onRefresh: _load,
        child: ListView(
          padding: const EdgeInsets.all(20),
          children: const [
            _EmptyStateCard(
              title: 'No saved problems yet',
              subtitle:
                  'Save one from the Rust picks on the home screen and it will appear here.',
            ),
          ],
        ),
      );
    }

    return RefreshIndicator(
      onRefresh: _load,
      child: ListView.separated(
        padding: const EdgeInsets.all(20),
        itemCount: _customProblems.length,
        separatorBuilder: (_, __) => const SizedBox(height: 12),
        itemBuilder: (context, index) {
          final item = _customProblems[index];
          return _SavedProblemTile(
            item: item,
            onTogglePinned: () => _togglePinned(item),
            onRecordDone: () => _recordDone(item),
            onAddNote: () => _showProblemNoteDialog(item),
          );
        },
      ),
    );
  }

  Widget _buildSnapshotTab() {
    if (_isLoading) {
      return const Center(child: CircularProgressIndicator());
    }
    if (_snapshots.isEmpty) {
      return RefreshIndicator(
        onRefresh: _load,
        child: ListView(
          padding: const EdgeInsets.all(20),
          children: const [
            _EmptyStateCard(
              title: 'No snapshots yet',
              subtitle:
                  'Capture one from the Rust dashboard on the home screen to keep a progress trail.',
            ),
          ],
        ),
      );
    }

    return RefreshIndicator(
      onRefresh: _load,
      child: ListView.separated(
        padding: const EdgeInsets.all(20),
        itemCount: _snapshots.length,
        separatorBuilder: (_, __) => const SizedBox(height: 12),
        itemBuilder: (context, index) {
          final snapshot = _snapshots[index];
          return _SnapshotTile(snapshot: snapshot);
        },
      ),
    );
  }
}

class _SavedProblemTile extends StatelessWidget {
  const _SavedProblemTile({
    required this.item,
    required this.onTogglePinned,
    required this.onRecordDone,
    required this.onAddNote,
  });

  final ProblemItem item;
  final VoidCallback onTogglePinned;
  final VoidCallback onRecordDone;
  final VoidCallback onAddNote;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(18),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Expanded(
                  child: Text(
                    item.title,
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                ),
                _MetaPill(label: item.mode),
              ],
            ),
            const SizedBox(height: 8),
            Text(item.prompt),
            const SizedBox(height: 10),
            Text(
              item.wmSupport,
              style: Theme.of(context).textTheme.bodySmall,
            ),
            const SizedBox(height: 10),
            Text(
              'Success check: ${item.successCheck}',
              style: Theme.of(context).textTheme.bodySmall,
            ),
            const SizedBox(height: 12),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                _MetaPill(label: item.levelBand),
                _MetaPill(label: item.targetContext),
                _MetaPill(label: item.topic),
                if (item.pinned) const _MetaPill(label: 'pinned'),
                _MetaPill(label: 'used ${item.usageCount}x'),
                _MetaPill(label: 'success ${item.successCount}x'),
              ],
            ),
            if (item.notes.isNotEmpty) ...[
              const SizedBox(height: 12),
              Text(item.notes),
            ],
            const SizedBox(height: 12),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                OutlinedButton.icon(
                  onPressed: onTogglePinned,
                  icon: Icon(
                    item.pinned ? Icons.push_pin : Icons.push_pin_outlined,
                  ),
                  label: Text(item.pinned ? 'Unpin' : 'Pin'),
                ),
                OutlinedButton.icon(
                  onPressed: onRecordDone,
                  icon: const Icon(Icons.check_circle_outline),
                  label: const Text('Done'),
                ),
                OutlinedButton.icon(
                  onPressed: onAddNote,
                  icon: const Icon(Icons.edit_note_outlined),
                  label: const Text('Note'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

class _SnapshotTile extends StatelessWidget {
  const _SnapshotTile({required this.snapshot});

  final RustProblemSnapshot snapshot;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(18),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Expanded(
                  child: Text(
                    _formatSnapshotDate(snapshot.capturedAtUnix),
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                ),
                _MetaPill(label: snapshot.riskLevel),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              'Next mode: ${snapshot.recommendedNextMode ?? 'reading'}',
              style: Theme.of(context).textTheme.bodyMedium,
            ),
            if (snapshot.note.isNotEmpty) ...[
              const SizedBox(height: 10),
              Text(snapshot.note),
            ],
          ],
        ),
      ),
    );
  }
}

class _EmptyStateCard extends StatelessWidget {
  const _EmptyStateCard({
    required this.title,
    required this.subtitle,
  });

  final String title;
  final String subtitle;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(18),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(title, style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 8),
            Text(subtitle),
          ],
        ),
      ),
    );
  }
}

class _MetaPill extends StatelessWidget {
  const _MetaPill({required this.label});

  final String label;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(999),
        color: Theme.of(context).colorScheme.secondaryContainer,
      ),
      child: Text(label),
    );
  }
}

String _formatSnapshotDate(int unixSeconds) {
  final dateTime = DateTime.fromMillisecondsSinceEpoch(unixSeconds * 1000);
  final month = dateTime.month.toString().padLeft(2, '0');
  final day = dateTime.day.toString().padLeft(2, '0');
  final hour = dateTime.hour.toString().padLeft(2, '0');
  final minute = dateTime.minute.toString().padLeft(2, '0');
  return '${dateTime.year}-$month-$day $hour:$minute';
}
