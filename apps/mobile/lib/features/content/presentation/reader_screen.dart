import 'package:flutter/material.dart';

import '../../../app/app_scope.dart';
import '../model/chunking_result.dart';
import '../model/content_item.dart';

enum ReaderMode { normal, chunk, skeleton, assisted }

class ReaderScreen extends StatefulWidget {
  const ReaderScreen({
    super.key,
    required this.contentId,
  });

  final String contentId;

  @override
  State<ReaderScreen> createState() => _ReaderScreenState();
}

class _ReaderScreenState extends State<ReaderScreen> {
  ContentItem? _content;
  ChunkingResult? _chunks;
  SkeletonResult? _skeleton;
  bool _isLoading = true;
  String? _errorText;
  ReaderMode _mode = ReaderMode.chunk;
  int _focusIndex = 0;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => _load());
  }

  Future<void> _load() async {
    final repository = AppScope.of(context).contentRepository;
    setState(() {
      _isLoading = true;
      _errorText = null;
    });

    try {
      final content = await repository.fetchContent(widget.contentId);
      final chunks = await repository.fetchChunks(widget.contentId);
      final skeleton = await repository.fetchSkeleton(widget.contentId);
      if (!mounted) {
        return;
      }

      setState(() {
        _content = content;
        _chunks = chunks;
        _skeleton = skeleton;
        _focusIndex = 0;
      });
    } catch (_) {
      if (mounted) {
        setState(() => _errorText = 'Failed to load content.');
      }
    } finally {
      if (mounted) {
        setState(() => _isLoading = false);
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final content = _content;
    final chunks = _chunks;
    final skeleton = _skeleton;

    return Scaffold(
      appBar: AppBar(title: const Text('Chunk Reader')),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(20),
          child: _isLoading
              ? const Center(child: CircularProgressIndicator())
              : _errorText != null
                  ? Center(child: Text(_errorText!))
                  : content == null || chunks == null || skeleton == null
                      ? const Center(child: Text('No content'))
                      : Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              content.title,
                              style: Theme.of(context).textTheme.headlineSmall,
                            ),
                            const SizedBox(height: 8),
                            Wrap(
                              spacing: 8,
                              runSpacing: 8,
                              children: [
                                _MetaChip(label: content.level),
                                _MetaChip(label: content.topic),
                                _MetaChip(
                                  label: content.language.toUpperCase(),
                                ),
                                if (chunks.version.isNotEmpty)
                                  _MetaChip(label: 'chunks ${chunks.version}'),
                                if (skeleton.version.isNotEmpty)
                                  _MetaChip(
                                    label: 'skeleton ${skeleton.version}',
                                  ),
                              ],
                            ),
                            const SizedBox(height: 16),
                            SegmentedButton<ReaderMode>(
                              segments: const [
                                ButtonSegment(
                                  value: ReaderMode.normal,
                                  label: Text('Normal'),
                                ),
                                ButtonSegment(
                                  value: ReaderMode.chunk,
                                  label: Text('Chunk'),
                                ),
                                ButtonSegment(
                                  value: ReaderMode.skeleton,
                                  label: Text('Skeleton'),
                                ),
                                ButtonSegment(
                                  value: ReaderMode.assisted,
                                  label: Text('Assisted'),
                                ),
                              ],
                              selected: {_mode},
                              onSelectionChanged: (selection) =>
                                  setState(() {
                                    _mode = selection.first;
                                    _focusIndex = 0;
                                  }),
                            ),
                            const SizedBox(height: 16),
                            if (_mode == ReaderMode.chunk ||
                                _mode == ReaderMode.assisted ||
                                _mode == ReaderMode.skeleton) ...[
                              _ReaderStepper(
                                currentIndex: _focusIndex,
                                totalCount: _mode == ReaderMode.skeleton
                                    ? skeleton.parts.length
                                    : chunks.chunks.length,
                                onPrevious: _focusIndex > 0
                                    ? () => setState(() => _focusIndex -= 1)
                                    : null,
                                onNext: _focusIndex <
                                        ((_mode == ReaderMode.skeleton
                                                ? skeleton.parts.length
                                                : chunks.chunks.length) -
                                            1)
                                    ? () => setState(() => _focusIndex += 1)
                                    : null,
                              ),
                              const SizedBox(height: 16),
                            ],
                            Expanded(
                              child: SingleChildScrollView(
                                child: _buildModeView(content, chunks, skeleton),
                              ),
                            ),
                          ],
                        ),
        ),
      ),
    );
  }

  Widget _buildModeView(
    ContentItem content,
    ChunkingResult chunks,
    SkeletonResult skeleton,
  ) {
    switch (_mode) {
      case ReaderMode.normal:
        return _TextCard(
          title: 'Original',
          body: content.rawText,
        );
      case ReaderMode.chunk:
        return _ChunkListView(
          chunks: chunks.chunks,
          dimSupport: false,
          onlyCore: false,
          focusIndex: _focusIndex,
        );
      case ReaderMode.skeleton:
        return _SkeletonListView(parts: skeleton.parts, focusIndex: _focusIndex);
      case ReaderMode.assisted:
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _TextCard(title: 'Skeleton Summary', body: skeleton.summary),
            const SizedBox(height: 12),
            _ChunkListView(
              chunks: chunks.chunks,
              dimSupport: true,
              onlyCore: false,
              focusIndex: _focusIndex,
            ),
          ],
        );
    }
  }
}

class _ChunkListView extends StatelessWidget {
  const _ChunkListView({
    required this.chunks,
    required this.dimSupport,
    required this.onlyCore,
    required this.focusIndex,
  });

  final List<ChunkItem> chunks;
  final bool dimSupport;
  final bool onlyCore;
  final int focusIndex;

  @override
  Widget build(BuildContext context) {
    final visible = onlyCore
        ? chunks.where((chunk) => chunk.isCore).toList()
        : chunks;
    return Column(
      children: [
        for (var index = 0; index < visible.length; index++) ...[
          final chunk = visible[index],
          Opacity(
            opacity: _resolveOpacity(index, chunk),
            child: Card(
              elevation: index == focusIndex ? 3 : 0,
              shape: RoundedRectangleBorder(
                side: BorderSide(
                  color: index == focusIndex
                      ? Theme.of(context).colorScheme.primary
                      : Colors.transparent,
                  width: 1.4,
                ),
                borderRadius: BorderRadius.circular(12),
              ),
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    CircleAvatar(
                      radius: 16,
                      backgroundColor: chunk.isCore
                          ? Theme.of(context).colorScheme.primaryContainer
                          : Theme.of(context).colorScheme.secondaryContainer,
                      child: Text(
                        '${chunk.order}',
                        style: Theme.of(context).textTheme.labelLarge,
                      ),
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            chunk.text,
                            style: Theme.of(context).textTheme.titleMedium,
                          ),
                          const SizedBox(height: 6),
                          Text(
                            chunk.role,
                            style: Theme.of(context)
                                .textTheme
                                .bodySmall
                                ?.copyWith(
                                  color: Theme.of(context).colorScheme.primary,
                                ),
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
          const SizedBox(height: 10),
        ],
      ],
    );
  }

  double _resolveOpacity(int index, ChunkItem chunk) {
    if (index == focusIndex) {
      return 1;
    }
    if (dimSupport && !chunk.isCore) {
      return 0.36;
    }
    if ((index - focusIndex).abs() == 1) {
      return 0.74;
    }
    return 0.42;
  }
}

class _SkeletonListView extends StatelessWidget {
  const _SkeletonListView({
    required this.parts,
    required this.focusIndex,
  });

  final List<SkeletonPart> parts;
  final int focusIndex;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        for (var index = 0; index < parts.length; index++) ...[
          final part = parts[index],
          Card(
            elevation: index == focusIndex ? 3 : 0,
            shape: RoundedRectangleBorder(
              side: BorderSide(
                color: index == focusIndex
                    ? Theme.of(context).colorScheme.primary
                    : Colors.transparent,
                width: 1.4,
              ),
              borderRadius: BorderRadius.circular(12),
            ),
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  CircleAvatar(
                    radius: 16,
                    backgroundColor: part.isCore
                        ? Theme.of(context).colorScheme.primary
                        : Theme.of(context).colorScheme.secondaryContainer,
                    foregroundColor: part.isCore ? Colors.white : null,
                    child: Text(
                      '${part.order}',
                      style: Theme.of(context).textTheme.labelLarge,
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          part.text,
                          style: Theme.of(context)
                              .textTheme
                              .titleMedium
                              ?.copyWith(
                                fontWeight: part.isCore
                                    ? FontWeight.w700
                                    : FontWeight.w500,
                              ),
                        ),
                        const SizedBox(height: 6),
                        Text(
                          '${part.role} • emphasis ${part.emphasis}',
                          style: Theme.of(context)
                              .textTheme
                              .bodySmall
                              ?.copyWith(
                                color: Theme.of(context).colorScheme.primary,
                              ),
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 10),
        ],
      ],
    );
  }
}

class _ReaderStepper extends StatelessWidget {
  const _ReaderStepper({
    required this.currentIndex,
    required this.totalCount,
    required this.onPrevious,
    required this.onNext,
  });

  final int currentIndex;
  final int totalCount;
  final VoidCallback? onPrevious;
  final VoidCallback? onNext;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        child: Row(
          children: [
            IconButton(
              onPressed: onPrevious,
              icon: const Icon(Icons.arrow_back_rounded),
            ),
            Expanded(
              child: Column(
                children: [
                  Text(
                    'Step ${currentIndex + 1} / $totalCount',
                    style: Theme.of(context).textTheme.titleSmall,
                  ),
                  const SizedBox(height: 8),
                  LinearProgressIndicator(
                    value: totalCount == 0 ? 0 : (currentIndex + 1) / totalCount,
                    minHeight: 8,
                    borderRadius: BorderRadius.circular(999),
                  ),
                ],
              ),
            ),
            IconButton(
              onPressed: onNext,
              icon: const Icon(Icons.arrow_forward_rounded),
            ),
          ],
        ),
      ),
    );
  }
}

class _TextCard extends StatelessWidget {
  const _TextCard({
    required this.title,
    required this.body,
  });

  final String title;
  final String body;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(18),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(title, style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 10),
            Text(
              body,
              style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                    height: 1.8,
                  ),
            ),
          ],
        ),
      ),
    );
  }
}

class _MetaChip extends StatelessWidget {
  const _MetaChip({required this.label});

  final String label;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(999),
      ),
      child: Text(label),
    );
  }
}
