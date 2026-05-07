class AuthSession {
  const AuthSession({
    required this.userId,
    required this.email,
    required this.displayName,
    required this.authProvider,
    required this.accessToken,
    required this.refreshToken,
    required this.nativeNotice,
  });

  final String userId;
  final String email;
  final String displayName;
  final String authProvider;
  final String accessToken;
  final String refreshToken;
  final String nativeNotice;

  bool get isUsable =>
      userId.isNotEmpty &&
      accessToken.isNotEmpty &&
      refreshToken.isNotEmpty;
  bool get isGuest => authProvider == 'guest';
  String get visibleEmail => isGuest ? 'Guest session on this device' : email;
  String get visibleDisplayName =>
      displayName.isNotEmpty ? displayName : (isGuest ? 'Guest learner' : 'Learner');
  String get guestNotice =>
      nativeNotice.isNotEmpty
          ? nativeNotice
          : 'You can explore the app without a password. Your session stays on this device.';

  factory AuthSession.fromJson(Map<String, dynamic> json) {
    final user = json['user'] as Map<String, dynamic>? ?? const <String, dynamic>{};
    final tokens = json['tokens'] as Map<String, dynamic>? ?? const <String, dynamic>{};
    return AuthSession(
      userId: user['user_id'] as String? ?? '',
      email: user['email'] as String? ?? '',
      displayName: user['display_name'] as String? ?? '',
      authProvider: user['auth_provider'] as String? ?? '',
      accessToken: tokens['access_token'] as String? ?? '',
      refreshToken: tokens['refresh_token'] as String? ?? '',
      nativeNotice: json['native_notice'] as String? ?? '',
    );
  }

  factory AuthSession.fromStorageJson(Map<String, dynamic> json) {
    return AuthSession(
      userId: json['user_id'] as String? ?? '',
      email: json['email'] as String? ?? '',
      displayName: json['display_name'] as String? ?? '',
      authProvider: json['auth_provider'] as String? ?? '',
      accessToken: json['access_token'] as String? ?? '',
      refreshToken: json['refresh_token'] as String? ?? '',
      nativeNotice: json['native_notice'] as String? ?? '',
    );
  }

  Map<String, dynamic> toStorageJson() {
    return <String, dynamic>{
      'user_id': userId,
      'email': email,
      'display_name': displayName,
      'auth_provider': authProvider,
      'access_token': accessToken,
      'refresh_token': refreshToken,
      'native_notice': nativeNotice,
    };
  }
}
