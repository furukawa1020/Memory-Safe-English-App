class AuthSession {
  const AuthSession({
    required this.userId,
    required this.email,
    required this.displayName,
    required this.accessToken,
    required this.refreshToken,
  });

  final String userId;
  final String email;
  final String displayName;
  final String accessToken;
  final String refreshToken;

  factory AuthSession.fromJson(Map<String, dynamic> json) {
    final user = json['user'] as Map<String, dynamic>? ?? const <String, dynamic>{};
    final tokens = json['tokens'] as Map<String, dynamic>? ?? const <String, dynamic>{};
    return AuthSession(
      userId: user['user_id'] as String? ?? '',
      email: user['email'] as String? ?? '',
      displayName: user['display_name'] as String? ?? '',
      accessToken: tokens['access_token'] as String? ?? '',
      refreshToken: tokens['refresh_token'] as String? ?? '',
    );
  }

  factory AuthSession.fromStorageJson(Map<String, dynamic> json) {
    return AuthSession(
      userId: json['user_id'] as String? ?? '',
      email: json['email'] as String? ?? '',
      displayName: json['display_name'] as String? ?? '',
      accessToken: json['access_token'] as String? ?? '',
      refreshToken: json['refresh_token'] as String? ?? '',
    );
  }

  Map<String, dynamic> toStorageJson() {
    return <String, dynamic>{
      'user_id': userId,
      'email': email,
      'display_name': displayName,
      'access_token': accessToken,
      'refresh_token': refreshToken,
    };
  }
}
