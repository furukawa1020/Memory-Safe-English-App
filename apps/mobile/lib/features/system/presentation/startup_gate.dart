import 'package:flutter/material.dart';

import '../../../app/app_scope.dart';
import '../model/backend_status.dart';
import 'startup_controller.dart';

class StartupGate extends StatefulWidget {
  const StartupGate({
    super.key,
    required this.controller,
    required this.readyChild,
  });

  final StartupController controller;
  final Widget readyChild;

  @override
  State<StartupGate> createState() => _StartupGateState();
}

class _StartupGateState extends State<StartupGate> {
  bool _allowBypass = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      widget.controller.load();
    });
  }

  @override
  Widget build(BuildContext context) {
    final apiBaseUrl = AppScope.of(context).config.apiBaseUrl;

    return AnimatedBuilder(
      animation: widget.controller,
      builder: (context, _) {
        final bootstrap = widget.controller.bootstrap;

        if (_allowBypass || widget.controller.isReady) {
          return widget.readyChild;
        }

        return Scaffold(
          body: SafeArea(
            child: Center(
              child: ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: 520),
                child: Padding(
                  padding: const EdgeInsets.all(24),
                  child: Card(
                    child: Padding(
                      padding: const EdgeInsets.all(24),
                      child: Column(
                        mainAxisSize: MainAxisSize.min,
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            'Preparing Memory-Safe English',
                            style: Theme.of(context).textTheme.headlineSmall,
                          ),
                          const SizedBox(height: 12),
                          Text(
                            'The mobile app waits for the local backend stack before continuing. '
                            'This keeps the first emulator launch much easier to debug.',
                            style: Theme.of(context).textTheme.bodyMedium,
                          ),
                          const SizedBox(height: 20),
                          if (widget.controller.isLoading)
                            const Row(
                              children: [
                                SizedBox(
                                  width: 20,
                                  height: 20,
                                  child: CircularProgressIndicator(strokeWidth: 2.4),
                                ),
                                SizedBox(width: 12),
                                Text('Checking backend readiness...'),
                              ],
                            )
                          else ...[
                            _StatusPanel(
                              title: 'Proxy target',
                              subtitle: apiBaseUrl,
                              healthy: widget.controller.backendStatus?.ready == true,
                            ),
                            const SizedBox(height: 12),
                            if (bootstrap != null) ...[
                              _InfoPanel(
                                title: 'Recommended Android emulator URL',
                                body: bootstrap.recommendedBaseUrls.androidEmulator,
                              ),
                              const SizedBox(height: 10),
                              _InfoPanel(
                                title: 'Frontend route map',
                                body:
                                    'login ${bootstrap.routes.login}\nrefresh ${bootstrap.routes.refresh}\ncontents ${bootstrap.routes.contents}\nanalyze ${bootstrap.routes.chunkAnalysis}',
                              ),
                              const SizedBox(height: 12),
                            ],
                            if (widget.controller.backendStatus != null) ...[
                              _UpstreamPanel(status: widget.controller.backendStatus!.api),
                              const SizedBox(height: 10),
                              _UpstreamPanel(status: widget.controller.backendStatus!.worker),
                            ] else if (widget.controller.errorText != null)
                              Text(
                                widget.controller.errorText!,
                                style: TextStyle(
                                  color: Theme.of(context).colorScheme.error,
                                ),
                              ),
                            const SizedBox(height: 16),
                            Text(
                              'Expected local command:',
                              style: Theme.of(context).textTheme.titleSmall,
                            ),
                            const SizedBox(height: 8),
                            const SelectableText(
                              'docker compose -f infra/docker-compose.yml up --build',
                            ),
                            const SizedBox(height: 18),
                            Row(
                              children: [
                                FilledButton(
                                  onPressed: widget.controller.load,
                                  child: const Text('Retry'),
                                ),
                                const SizedBox(width: 12),
                                FilledButton.tonal(
                                  onPressed: () => setState(() => _allowBypass = true),
                                  child: const Text('Continue anyway'),
                                ),
                              ],
                            ),
                          ],
                        ],
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}

class _InfoPanel extends StatelessWidget {
  const _InfoPanel({
    required this.title,
    required this.body,
  });

  final String title;
  final String body;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(title, style: Theme.of(context).textTheme.titleSmall),
            const SizedBox(height: 6),
            SelectableText(body),
          ],
        ),
      ),
    );
  }
}

class _StatusPanel extends StatelessWidget {
  const _StatusPanel({
    required this.title,
    required this.subtitle,
    required this.healthy,
  });

  final String title;
  final String subtitle;
  final bool healthy;

  @override
  Widget build(BuildContext context) {
    return DecoratedBox(
      decoration: BoxDecoration(
        color: healthy
            ? Theme.of(context).colorScheme.primaryContainer
            : Theme.of(context).colorScheme.errorContainer,
        borderRadius: BorderRadius.circular(20),
      ),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            Icon(
              healthy ? Icons.check_circle_outline : Icons.error_outline,
              color: healthy
                  ? Theme.of(context).colorScheme.primary
                  : Theme.of(context).colorScheme.error,
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(title, style: Theme.of(context).textTheme.titleSmall),
                  const SizedBox(height: 4),
                  Text(subtitle, style: Theme.of(context).textTheme.bodyMedium),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _UpstreamPanel extends StatelessWidget {
  const _UpstreamPanel({required this.status});

  final UpstreamStatus status;

  @override
  Widget build(BuildContext context) {
    return _StatusPanel(
      title: status.name.toUpperCase(),
      subtitle: '${status.url}  (${status.statusCode == 0 ? 'no response' : status.statusCode})',
      healthy: status.ok,
    );
  }
}
