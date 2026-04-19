import 'package:flutter/material.dart';

import '../../../app/app_scope.dart';
import '../../../app/session_controller.dart';
import '../../analysis/presentation/analysis_controller.dart';
import '../model/chunking_result.dart';
import '../model/content_item.dart';
import 'content_catalog_controller.dart';
import 'reader_screen.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  int _index = 0;

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    final pages = [
      _ContentHomeTab(controller: ContentCatalogController(scope.contentRepository)),
      _AnalysisTab(controller: AnalysisController(scope.contentRepository)),
      _SettingsTab(sessionController: scope.sessionController),
    ];

    return Scaffold(
      body: pages[_index],
      bottomNavigationBar: NavigationBar(
        selectedIndex: _index,
        onDestinationSelected: (value) => setState(() => _index = value),
        destinations: const [
          NavigationDestination(icon: Icon(Icons.menu_book_outlined), label: 'Read'),
          NavigationDestination(icon: Icon(Icons.tune_outlined), label: 'Analyze'),
          NavigationDestination(icon: Icon(Icons.settings_outlined), label: 'Settings'),
        ],
      ),
    );
  }
}

class _ContentHomeTab extends StatefulWidget {
  const _ContentHomeTab({required this.controller});

  final ContentCatalogController controller;

  @override
  State<_ContentHomeTab> createState() => _ContentHomeTabState();
}

class _ContentHomeTabState extends State<_ContentHomeTab> {
  @override
  void initState() {
    super.initState();
    widget.controller.load();
  }

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: AnimatedBuilder(
        animation: widget.controller,
        builder: (context, _) {
          return RefreshIndicator(
            onRefresh: widget.controller.load,
            child: ListView(
              padding: const EdgeInsets.all(20),
              children: [
                const _HeroCard(
                  title: 'Chunk Reader',
                  subtitle: 'Read with meaning chunks kept visible so sentence-level load stays lower.',
                ),
                const SizedBox(height: 18),
                Text('Recommended Content', style: Theme.of(context).textTheme.titleLarge),
                const SizedBox(height: 12),
                if (widget.controller.isLoading)
                  const Center(child: Padding(padding: EdgeInsets.all(32), child: CircularProgressIndicator()))
                else if (widget.controller.errorText != null)
                  Text(widget.controller.errorText!)
                else
                  for (final item in widget.controller.items) ...[
                    _ContentTile(item: item),
                    const SizedBox(height: 12),
                  ],
              ],
            ),
          );
        },
      ),
    );
  }
}

class _AnalysisTab extends StatefulWidget {
  const _AnalysisTab({required this.controller});

  final AnalysisController controller;

  @override
  State<_AnalysisTab> createState() => _AnalysisTabState();
}

class _AnalysisTabState extends State<_AnalysisTab> {
  final _textController = TextEditingController(
    text: 'In this study, we propose a memory safe interface that reduces cognitive overload during English reading.',
  );

  @override
  void dispose() {
    _textController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: AnimatedBuilder(
        animation: widget.controller,
        builder: (context, _) {
          final result = widget.controller.result;
          return ListView(
            padding: const EdgeInsets.all(20),
            children: [
              const _HeroCard(
                title: 'Free Analysis',
                subtitle: 'Paste English text and inspect chunk boundaries before reading it in full.',
              ),
              const SizedBox(height: 16),
              TextField(
                controller: _textController,
                minLines: 5,
                maxLines: 8,
                decoration: const InputDecoration(
                  labelText: 'English text',
                  alignLabelWithHint: true,
                ),
              ),
              const SizedBox(height: 12),
              FilledButton(
                onPressed: widget.controller.isSubmitting ? null : () => widget.controller.analyze(_textController.text),
                child: Padding(
                  padding: const EdgeInsets.symmetric(vertical: 12),
                  child: Text(widget.controller.isSubmitting ? 'Analyzing...' : 'Analyze chunks'),
                ),
              ),
              if (widget.controller.errorText != null) ...[
                const SizedBox(height: 12),
                Text(widget.controller.errorText!, style: TextStyle(color: Theme.of(context).colorScheme.error)),
              ],
              if (result != null) ...[
                const SizedBox(height: 18),
                Text(result.summary, style: Theme.of(context).textTheme.titleMedium),
                const SizedBox(height: 12),
                for (final chunk in result.chunks) ...[
                  _AnalysisChunkCard(chunk: chunk),
                  const SizedBox(height: 10),
                ],
              ],
            ],
          );
        },
      ),
    );
  }
}

class _SettingsTab extends StatelessWidget {
  const _SettingsTab({required this.sessionController});

  final SessionController sessionController;

  @override
  Widget build(BuildContext context) {
    final session = sessionController.session;
    return SafeArea(
      child: ListView(
        padding: const EdgeInsets.all(20),
        children: [
          const _HeroCard(
            title: 'Settings',
            subtitle: 'Keep session and connection details simple for now, then extend into accessibility controls.',
          ),
          const SizedBox(height: 16),
          Card(
            child: Padding(
              padding: const EdgeInsets.all(18),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(session?.displayName ?? '', style: Theme.of(context).textTheme.titleLarge),
                  const SizedBox(height: 6),
                  Text(session?.email ?? ''),
                  const SizedBox(height: 14),
                  FilledButton.tonal(
                    onPressed: sessionController.logout,
                    child: const Text('Logout'),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _ContentTile extends StatelessWidget {
  const _ContentTile({required this.item});

  final ContentItem item;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      borderRadius: BorderRadius.circular(24),
      onTap: () {
        Navigator.of(context).push(
          MaterialPageRoute<void>(
            builder: (_) => ReaderScreen(contentId: item.contentId),
          ),
        );
      },
      child: Card(
        child: Padding(
          padding: const EdgeInsets.all(18),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(item.title, style: Theme.of(context).textTheme.titleMedium),
              const SizedBox(height: 8),
              Text(item.summaryText),
              const SizedBox(height: 14),
              Wrap(
                spacing: 8,
                runSpacing: 8,
                children: [
                  _Pill(label: item.level),
                  _Pill(label: item.topic),
                  _Pill(label: item.contentType),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _AnalysisChunkCard extends StatelessWidget {
  const _AnalysisChunkCard({required this.chunk});

  final ChunkItem chunk;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            _Pill(label: chunk.role),
            const SizedBox(width: 12),
            Expanded(child: Text(chunk.text)),
          ],
        ),
      ),
    );
  }
}

class _HeroCard extends StatelessWidget {
  const _HeroCard({
    required this.title,
    required this.subtitle,
  });

  final String title;
  final String subtitle;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(22),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(28),
        gradient: const LinearGradient(
          colors: [Color(0xFF195D54), Color(0xFF3A8A76)],
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            title,
            style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                  color: Colors.white,
                  fontWeight: FontWeight.w700,
                ),
          ),
          const SizedBox(height: 8),
          Text(
            subtitle,
            style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                  color: Colors.white.withOpacity(0.9),
                  height: 1.5,
                ),
          ),
        ],
      ),
    );
  }
}

class _Pill extends StatelessWidget {
  const _Pill({required this.label});

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
