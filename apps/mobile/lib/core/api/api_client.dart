import 'dart:convert';

import 'package:http/http.dart' as http;

import '../../app/session_controller.dart';
import 'api_exception.dart';
import 'api_response.dart';

class ApiClient {
  ApiClient({
    required this.baseUrl,
    required this.sessionController,
    http.Client? httpClient,
  }) : _httpClient = httpClient ?? http.Client();

  final String baseUrl;
  final SessionController sessionController;
  final http.Client _httpClient;

  Future<JsonMap> get(String path, {bool authenticated = true}) async {
    final response = await getResponse(path, authenticated: authenticated);
    _throwIfFailed(response);
    return response.body;
  }

  Future<JsonMap> post(
    String path, {
    JsonMap? body,
    bool authenticated = true,
  }) async {
    final response = await postResponse(
      path,
      body: body,
      authenticated: authenticated,
    );
    _throwIfFailed(response);
    return response.body;
  }

  Future<JsonMap> patch(
    String path, {
    JsonMap? body,
    bool authenticated = true,
  }) async {
    final response = await patchResponse(
      path,
      body: body,
      authenticated: authenticated,
    );
    _throwIfFailed(response);
    return response.body;
  }

  Future<ApiResponse> getResponse(
    String path, {
    bool authenticated = true,
  }) {
    return _send('GET', path, authenticated: authenticated);
  }

  Future<ApiResponse> postResponse(
    String path, {
    JsonMap? body,
    bool authenticated = true,
  }) {
    return _send(
      'POST',
      path,
      body: body,
      authenticated: authenticated,
    );
  }

  Future<ApiResponse> patchResponse(
    String path, {
    JsonMap? body,
    bool authenticated = true,
  }) {
    return _send(
      'PATCH',
      path,
      body: body,
      authenticated: authenticated,
    );
  }

  Future<ApiResponse> _send(
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
        response = await _httpClient.post(
          uri,
          headers: headers,
          body: jsonEncode(body ?? <String, dynamic>{}),
        );
      case 'PATCH':
        response = await _httpClient.patch(
          uri,
          headers: headers,
          body: jsonEncode(body ?? <String, dynamic>{}),
        );
      default:
        throw UnsupportedError('Unsupported method: $method');
    }

    final decodedBody = _decodeBody(response);
    final apiResponse = ApiResponse(
      statusCode: response.statusCode,
      body: decodedBody,
    );

    return apiResponse;
  }

  JsonMap _decodeBody(http.Response response) {
    if (response.body.isEmpty) {
      return <String, dynamic>{};
    }

    final dynamic decoded;
    try {
      decoded = jsonDecode(response.body);
    } on FormatException {
      throw ApiException(
        statusCode: response.statusCode,
        code: 'invalid_json_response',
        message: 'The server returned an unreadable response.',
      );
    }

    if (decoded is Map<String, dynamic>) {
      return decoded;
    }
    if (decoded is Map) {
      return decoded.map(
        (key, value) => MapEntry(key.toString(), value),
      );
    }

    return <String, dynamic>{'value': decoded};
  }

  ApiException _buildApiException(ApiResponse response) {
    final errorField = response.body['error'];
    if (errorField is Map<String, dynamic>) {
      return ApiException(
        statusCode: response.statusCode,
        code: errorField['code'] as String? ?? 'request_failed',
        message: errorField['message'] as String? ?? 'Request failed.',
      );
    }
    if (errorField is Map) {
      return ApiException(
        statusCode: response.statusCode,
        code: errorField['code']?.toString() ?? 'request_failed',
        message: errorField['message']?.toString() ?? 'Request failed.',
      );
    }
    if (errorField is String && errorField.isNotEmpty) {
      return ApiException(
        statusCode: response.statusCode,
        code: 'request_failed',
        message: errorField,
      );
    }

    return ApiException(
      statusCode: response.statusCode,
      code: 'request_failed',
      message: 'Request failed with status ${response.statusCode}.',
    );
  }

  void _throwIfFailed(ApiResponse response) {
    if (!response.isSuccess) {
      throw _buildApiException(response);
    }
  }
}
