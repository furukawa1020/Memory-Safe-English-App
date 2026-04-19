class BackendStatus {
  const BackendStatus({
    required this.ready,
    required this.checkedAtUnixMs,
    required this.api,
    required this.worker,
  });

  final bool ready;
  final int checkedAtUnixMs;
  final UpstreamStatus api;
  final UpstreamStatus worker;

  factory BackendStatus.fromJson(Map<String, dynamic> json) {
    return BackendStatus(
      ready: json['ready'] as bool? ?? false,
      checkedAtUnixMs: _readInt(json['checked_at_unix_ms']),
      api: UpstreamStatus.fromJson(
        json['api'] as Map<String, dynamic>? ?? const <String, dynamic>{},
        fallbackName: 'api',
      ),
      worker: UpstreamStatus.fromJson(
        json['worker'] as Map<String, dynamic>? ?? const <String, dynamic>{},
        fallbackName: 'worker',
      ),
    );
  }

  static int _readInt(Object? value) {
    if (value is int) {
      return value;
    }
    if (value is num) {
      return value.toInt();
    }
    return 0;
  }
}

class UpstreamStatus {
  const UpstreamStatus({
    required this.name,
    required this.ok,
    required this.statusCode,
    required this.url,
  });

  final String name;
  final bool ok;
  final int statusCode;
  final String url;

  factory UpstreamStatus.fromJson(
    Map<String, dynamic> json, {
    required String fallbackName,
  }) {
    return UpstreamStatus(
      name: json['name'] as String? ?? fallbackName,
      ok: json['ok'] as bool? ?? false,
      statusCode: BackendStatus._readInt(json['status_code']),
      url: json['url'] as String? ?? '',
    );
  }
}
