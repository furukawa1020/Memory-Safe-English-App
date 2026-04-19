import '../../../core/api/api_client.dart';
import '../../../core/api/api_exception.dart';
import '../model/backend_status.dart';

class SystemRepository {
  const SystemRepository(this._apiClient);

  final ApiClient _apiClient;

  Future<BackendStatus> fetchBackendStatus() async {
    final response = await _apiClient.getResponse(
      '/ready',
      authenticated: false,
    );

    if (response.statusCode == 200 || response.statusCode == 503) {
      return BackendStatus.fromJson(response.body);
    }

    throw ApiException(
      statusCode: response.statusCode,
      code: 'backend_status_failed',
      message: 'Could not read backend readiness.',
    );
  }
}
