import 'dart:convert';

import 'package:http/http.dart' as http;

import '../../app/session_controller.dart';
import 'api_exception.dart';

typedef JsonMap = Map<String, dynamic>;

class ApiClient {
  ApiClient({
    required this.baseUrl,
    required this.sessionController,
    http.Client? httpClient,
  }) : _httpClient = httpClient ?? http.Client();

  final String baseUrl;
  final SessionController sessionController;
  final http.Client _httpClient;

  Future<JsonMap> get(String path) {
    return _send('GET', path);
  }

  Future<JsonMap> post(String path, {JsonMap? body, bool authenticated = true}) {
    return _send('POST', path, body: body, authenticated: authenticated);
  }

  Future<JsonMap> patch(String path, {JsonMap? body, bool authenticated = true}) {
    return _send('PATCH', path, body: body, authenticated: authenticated);
  }

  Future<JsonMap> _send(
    String method,
    String path, {
    JsonMap? body,
    bool authenticated = true,
  }) async {
    final uri = Uri.parse('$baseUrl$path');
    final headers = <String, String>{
      'Content-Type': 'application/json',
      'Accept': 'application/json',
    };
    final accessToken = sessionController.accessToken;
    if (authenticated && accessToken != null && accessToken.isNotEmpty) {
      headers['Authorization'] = 'Bearer $accessToken';
    }

    late final http.Response response;
    switch (method) {
      case 'GET':
        response = await _httpClient.get(uri, headers: headers);
      case 'POST':
        response = await _httpClient.post(uri, headers: headers, body: jsonEncode(body ?? <String, dynamic>{}));
      case 'PATCH':
        response = await _httpClient.patch(uri, headers: headers, body: jsonEncode(body ?? <String, dynamic>{}));
      default:
        throw UnsupportedError('Unsupported method: $method');
    }

    final decoded = response.body.isEmpty ? <String, dynamic>{} : jsonDecode(response.body) as JsonMap;
    if (response.statusCode >= 200 && response.statusCode < 300) {
      return decoded;
    }

    final errorMap = decoded['error'] as Map<String, dynamic>?;
    throw ApiException(
      statusCode: response.statusCode,
      code: errorMap?['code'] as String? ?? 'unknown_error',
      message: errorMap?['message'] as String? ?? 'Request failed',
    );
  }
}
