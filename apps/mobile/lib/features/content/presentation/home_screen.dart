import 'package:flutter/material.dart';

import '../../../app/app_scope.dart';
import '../../../app/session_controller.dart';
import '../../analysis/presentation/analysis_controller.dart';
import '../model/chunking_result.dart';
import '../model/content_item.dart';
import '../model/problem_item.dart';
import 'content_catalog_controller.dart';
import 'reader_screen.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  int _index = 0;
  ContentCatalogController? _contentCatalogController;
  AnalysisController? _analysisController;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final scope = AppScope.of(context);
    _contentCatalogController ??=
        ContentCatalogController(scope.contentRepository);
    _analysisController ??= AnalysisController(scope.contentRepository);
  }

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    final contentCatalogController = _contentCatalogController!;
    final analysisController = _analysisController!;
    final pages = [
      _ContentHomeTab(
        controller: contentCatalogController,
        onOpenListening: () {
          analysisController.setMode(AnalysisMode.listening);
          setState(() => _index = 1);
        },
        onOpenSpeaking: () {
          analysisController.setMode(AnalysisMode.speaking);
          setState(() => _index = 1);
        },
      ),
      _AnalysisTab(controller: analysisController),
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
  const _ContentHomeTab({
    required this.controller,
    required this.onOpenListening,
    required this.onOpenSpeaking,
  });

  final ContentCatalogController controller;
  final VoidCallback onOpenListening;
  final VoidCallback onOpenSpeaking;

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
    final groupedSections = _buildContentSections(widget.controller.items);
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
                  subtitle: 'Keep the sentence stable by reading one meaning unit at a time instead of holding the whole line in memory.',
                ),
                const SizedBox(height: 18),
                _PracticeDoorwaysPanel(
                  onOpenListening: widget.onOpenListening,
                  onOpenSpeaking: widget.onOpenSpeaking,
                ),
                const SizedBox(height: 18),
                const _QuickStartPanel(),
                const SizedBox(height: 18),
                Text('Rust Problem Picks', style: Theme.of(context).textTheme.titleLarge),
                const SizedBox(height: 12),
                if (!widget.controller.isLoading &&
                    widget.controller.recommendedItems.isNotEmpty) ...[
                  for (final item in widget.controller.recommendedItems) ...[
                    _ProblemTile(item: item),
                    const SizedBox(height: 12),
                  ],
                  const SizedBox(height: 8),
                ],
                Text('Recommended Content', style: Theme.of(context).textTheme.titleLarge),
                const SizedBox(height: 12),
                if (widget.controller.isLoading)
                  const Center(child: Padding(padding: EdgeInsets.all(32), child: CircularProgressIndicator()))
                else if (widget.controller.errorText != null)
                  Text(widget.controller.errorText!)
                else
                  for (final section in groupedSections) ...[
                    _SectionHeader(
                      title: section.title,
                      subtitle: section.subtitle,
                    ),
                    const SizedBox(height: 12),
                    for (final item in section.items) ...[
                      _ContentTile(item: item),
                      const SizedBox(height: 12),
                    ],
                    const SizedBox(height: 8),
                  ],
              ],
            ),
          );
        },
      ),
    );
  }

  List<_ContentSection> _buildContentSections(List<ContentItem> items) {
    final intro = <ContentItem>[];
    final toeic600to700 = <ContentItem>[];
    final toeic750to800 = <ContentItem>[];

    for (final item in items) {
      switch (item.level) {
        case 'intro':
          intro.add(item);
          break;
        case 'intermediate':
          toeic600to700.add(item);
          break;
        case 'upper_intermediate':
          toeic750to800.add(item);
          break;
        default:
          toeic600to700.add(item);
          break;
      }
    }

    final sections = <_ContentSection>[];
    if (intro.isNotEmpty) {
      sections.add(
        _ContentSection(
          title: 'Start Here',
          subtitle: 'Shorter, lower-load items for warming up before heavier business English.',
          items: intro,
        ),
      );
    }
    if (toeic600to700.isNotEmpty) {
      sections.add(
        _ContentSection(
          title: 'TOEIC 600-700 Bridge',
          subtitle: 'Intermediate items for keeping the main sentence stable across daily, meeting, and research contexts.',
          items: toeic600to700,
        ),
      );
    }
    if (toeic750to800.isNotEmpty) {
      sections.add(
        _ContentSection(
          title: 'TOEIC 750-800 Business Track',
          subtitle: 'Upper-intermediate notices, meetings, procedures, and updates with stronger business vocabulary.',
          items: toeic750to800,
        ),
      );
    }
    return sections;
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
          final chunkResult = widget.controller.chunkResult;
          final listeningResult = widget.controller.listeningResult;
          final speakingResult = widget.controller.speakingResult;
          return ListView(
            padding: const EdgeInsets.all(20),
            children: [
              const _HeroCard(
                title: 'Free Analysis',
                subtitle: 'Paste English text and switch between reading, listening, and speaking support before you try the full task.',
              ),
              const SizedBox(height: 16),
              SegmentedButton<AnalysisMode>(
                segments: const [
                  ButtonSegment(
                    value: AnalysisMode.chunks,
                    icon: Icon(Icons.view_stream_outlined),
                    label: Text('Chunks'),
                  ),
                  ButtonSegment(
                    value: AnalysisMode.listening,
                    icon: Icon(Icons.hearing_outlined),
                    label: Text('Listening'),
                  ),
                  ButtonSegment(
                    value: AnalysisMode.speaking,
                    icon: Icon(Icons.record_voice_over_outlined),
                    label: Text('Speaking'),
                  ),
                ],
                selected: {widget.controller.mode},
                onSelectionChanged: (selection) =>
                    widget.controller.setMode(selection.first),
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
                  child: Text(
                    widget.controller.isSubmitting
                        ? 'Analyzing...'
                        : switch (widget.controller.mode) {
                            AnalysisMode.chunks => 'Analyze chunks',
                            AnalysisMode.listening => 'Build listening plan',
                            AnalysisMode.speaking => 'Build speaking plan',
                          },
                  ),
                ),
              ),
              if (widget.controller.errorText != null) ...[
                const SizedBox(height: 12),
                Text(widget.controller.errorText!, style: TextStyle(color: Theme.of(context).colorScheme.error)),
              ],
              if (chunkResult != null &&
                  widget.controller.mode == AnalysisMode.chunks) ...[
                const SizedBox(height: 18),
                Text(chunkResult.summary, style: Theme.of(context).textTheme.titleMedium),
                const SizedBox(height: 12),
                for (final chunk in chunkResult.chunks) ...[
                  _AnalysisChunkCard(chunk: chunk),
                  const SizedBox(height: 10),
                ],
              ],
              if (listeningResult != null &&
                  widget.controller.mode == AnalysisMode.listening) ...[
                const SizedBox(height: 18),
                _ListeningPlanCard(result: listeningResult),
              ],
              if (speakingResult != null &&
                  widget.controller.mode == AnalysisMode.speaking) ...[
                const SizedBox(height: 18),
                _SpeakingPlanCard(result: speakingResult),
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
              Row(
                children: [
                  Expanded(
                    child: Text(item.title, style: Theme.of(context).textTheme.titleMedium),
                  ),
                  const Icon(Icons.arrow_forward_rounded),
                ],
              ),
              const SizedBox(height: 8),
              Text(item.summaryText),
              const SizedBox(height: 14),
              Wrap(
                spacing: 8,
                runSpacing: 8,
                children: [
                  _Pill(label: _levelLabel(item.level)),
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

class _ProblemTile extends StatelessWidget {
  const _ProblemTile({required this.item});

  final ProblemItem item;

  @override
  Widget build(BuildContext context) {
    return Card(
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
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
                _Pill(label: item.mode),
              ],
            ),
            const SizedBox(height: 8),
            Text(item.prompt),
            const SizedBox(height: 12),
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
                _Pill(label: item.levelBand),
                _Pill(label: item.targetContext),
                _Pill(label: item.topic),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

class _ContentSection {
  const _ContentSection({
    required this.title,
    required this.subtitle,
    required this.items,
  });

  final String title;
  final String subtitle;
  final List<ContentItem> items;
}

class _SectionHeader extends StatelessWidget {
  const _SectionHeader({
    required this.title,
    required this.subtitle,
  });

  final String title;
  final String subtitle;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(title, style: Theme.of(context).textTheme.titleMedium),
        const SizedBox(height: 4),
        Text(subtitle, style: Theme.of(context).textTheme.bodySmall),
      ],
    );
  }
}

class _PracticeDoorwaysPanel extends StatelessWidget {
  const _PracticeDoorwaysPanel({
    required this.onOpenListening,
    required this.onOpenSpeaking,
  });

  final VoidCallback onOpenListening;
  final VoidCallback onOpenSpeaking;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Open a support mode directly', style: Theme.of(context).textTheme.titleLarge),
        const SizedBox(height: 12),
        Row(
          children: [
            Expanded(
              child: _DoorwayCard(
                icon: Icons.hearing_outlined,
                title: 'Listening plan',
                subtitle: 'Open pause points and replay cues first.',
                onTap: onOpenListening,
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: _DoorwayCard(
                icon: Icons.record_voice_over_outlined,
                title: 'Speaking plan',
                subtitle: 'Open short speaking steps and rescue phrases.',
                onTap: onOpenSpeaking,
              ),
            ),
          ],
        ),
      ],
    );
  }
}

class _DoorwayCard extends StatelessWidget {
  const _DoorwayCard({
    required this.icon,
    required this.title,
    required this.subtitle,
    required this.onTap,
  });

  final IconData icon;
  final String title;
  final String subtitle;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      borderRadius: BorderRadius.circular(24),
      onTap: onTap,
      child: Card(
        child: Padding(
          padding: const EdgeInsets.all(18),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Icon(icon, color: Theme.of(context).colorScheme.primary),
              const SizedBox(height: 12),
              Text(title, style: Theme.of(context).textTheme.titleMedium),
              const SizedBox(height: 8),
              Text(subtitle, style: Theme.of(context).textTheme.bodySmall),
              const SizedBox(height: 16),
              Text(
                'Open now',
                style: Theme.of(context).textTheme.labelLarge?.copyWith(
                      color: Theme.of(context).colorScheme.primary,
                    ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _QuickStartPanel extends StatelessWidget {
  const _QuickStartPanel();

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(18),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Today\'s low-load flow', style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 10),
            const _FlowStep(
              index: 1,
              title: 'Open one short text',
              subtitle: 'Start with a single item instead of choosing too many tasks at once.',
            ),
            const SizedBox(height: 10),
            const _FlowStep(
              index: 2,
              title: 'Use Chunk or Assisted first',
              subtitle: 'Keep support detail dimmed until the core meaning feels stable.',
            ),
            const SizedBox(height: 10),
            const _FlowStep(
              index: 3,
              title: 'Move to Skeleton only after the main idea lands',
              subtitle: 'Do not force yourself to hold every modifier from the start.',
            ),
          ],
        ),
      ),
    );
  }
}

class _FlowStep extends StatelessWidget {
  const _FlowStep({
    required this.index,
    required this.title,
    required this.subtitle,
  });

  final int index;
  final String title;
  final String subtitle;

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        CircleAvatar(
          radius: 16,
          backgroundColor: Theme.of(context).colorScheme.primaryContainer,
          child: Text('$index'),
        ),
        const SizedBox(width: 12),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(title, style: Theme.of(context).textTheme.titleSmall),
              const SizedBox(height: 4),
              Text(subtitle, style: Theme.of(context).textTheme.bodySmall),
            ],
          ),
        ),
      ],
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

class _ListeningPlanCard extends StatelessWidget {
  const _ListeningPlanCard({required this.result});

  final ListeningPlanResult result;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(18),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(result.summary, style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 8),
            Text('Recommended speed: ${result.recommendedSpeed}'),
            const SizedBox(height: 8),
            Text(result.finalPassStrategy),
            const SizedBox(height: 14),
            for (final point in result.pausePoints) ...[
              _PlanSectionLabel(
                title: 'Pause ${point.index}',
                subtitle: '${point.pauseReason} • risk ${point.riskLevel}',
              ),
              const SizedBox(height: 6),
              Text(point.previewText),
              const SizedBox(height: 4),
              Text(point.cueJa),
              const SizedBox(height: 12),
            ],
          ],
        ),
      ),
    );
  }
}

class _SpeakingPlanCard extends StatelessWidget {
  const _SpeakingPlanCard({required this.result});

  final SpeakingPlanResult result;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(18),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(result.summary, style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 8),
            Text('Recommended style: ${result.recommendedStyle}'),
            if (result.openerOptions.isNotEmpty) ...[
              const SizedBox(height: 12),
              const _PlanSectionLabel(
                title: 'Openers',
                subtitle: 'Use one of these to start without holding the whole answer.',
              ),
              const SizedBox(height: 6),
              for (final opener in result.openerOptions) ...[
                Text('• $opener'),
                const SizedBox(height: 4),
              ],
            ],
            if (result.steps.isNotEmpty) ...[
              const SizedBox(height: 12),
              const _PlanSectionLabel(
                title: 'Speaking steps',
                subtitle: 'Keep each step short and stable.',
              ),
              const SizedBox(height: 6),
              for (final step in result.steps) ...[
                Card(
                  elevation: 0,
                  color: Theme.of(context).colorScheme.surfaceContainerHighest,
                  child: Padding(
                    padding: const EdgeInsets.all(14),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('${step.step}. ${step.text}'),
                        const SizedBox(height: 6),
                        Text(
                          '${step.purpose} • risk ${step.riskLevel}',
                          style: Theme.of(context).textTheme.bodySmall,
                        ),
                        const SizedBox(height: 6),
                        Text(step.deliveryTipJa),
                      ],
                    ),
                  ),
                ),
                const SizedBox(height: 8),
              ],
            ],
            if (result.rescuePhrases.isNotEmpty) ...[
              const SizedBox(height: 12),
              const _PlanSectionLabel(
                title: 'Rescue phrases',
                subtitle: 'Fallback lines for when the sentence starts to collapse.',
              ),
              const SizedBox(height: 6),
              for (final phrase in result.rescuePhrases) ...[
                Text('• $phrase'),
                const SizedBox(height: 4),
              ],
            ],
          ],
        ),
      ),
    );
  }
}

class _PlanSectionLabel extends StatelessWidget {
  const _PlanSectionLabel({
    required this.title,
    required this.subtitle,
  });

  final String title;
  final String subtitle;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(title, style: Theme.of(context).textTheme.titleSmall),
        const SizedBox(height: 2),
        Text(subtitle, style: Theme.of(context).textTheme.bodySmall),
      ],
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

String _levelLabel(String level) {
  switch (level) {
    case 'intro':
      return 'starter';
    case 'intermediate':
      return 'toeic 600-700';
    case 'upper_intermediate':
      return 'toeic 750-800';
    default:
      return level;
  }
}
