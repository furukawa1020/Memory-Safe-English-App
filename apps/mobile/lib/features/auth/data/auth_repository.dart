import '../../../core/api/api_client.dart';
import '../model/auth_session.dart';

class AuthRepository {
  AuthRepository(this._apiClient);

  final ApiClient _apiClient;

  Future<AuthSession> register({
    required String email,
    required String password,
    required String displayName,
  }) async {
    final response = await _apiClient.post(
      '/auth/register',
      authenticated: false,
      body: <String, dynamic>{
        'email': email,
        'password': password,
        'display_name': displayName,
        'agreed_to_terms': true,
      },
    );
    return AuthSession.fromJson(response);
  }

  Future<AuthSession> login({
    required String email,
    required String password,
  }) async {
    final response = await _apiClient.post(
      '/auth/login',
      authenticated: false,
      body: <String, dynamic>{
        'email': email,
        'password': password,
      },
    );
    return AuthSession.fromJson(response);
  }

  Future<AuthSession> continueAsGuest() async {
    final response = await _apiClient.post(
      '/auth/guest',
      authenticated: false,
      body: const <String, dynamic>{},
    );
    return AuthSession.fromJson(response);
  }

  Future<AuthSession> refreshSession(String refreshToken) async {
    final response = await _apiClient.post(
      '/auth/refresh',
      authenticated: false,
      body: <String, dynamic>{
        'refresh_token': refreshToken,
      },
    );
    return AuthSession.fromJson(response);
  }
}
