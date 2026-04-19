class AppConfig {
  const AppConfig({
    required this.apiBaseUrl,
  });

  final String apiBaseUrl;

  factory AppConfig.fromEnvironment() {
    const rawBaseUrl = String.fromEnvironment(
      'API_BASE_URL',
      defaultValue: 'http://10.0.2.2:8070',
    );

    return AppConfig(
      apiBaseUrl: rawBaseUrl.endsWith('/')
          ? rawBaseUrl.substring(0, rawBaseUrl.length - 1)
          : rawBaseUrl,
    );
  }
}
