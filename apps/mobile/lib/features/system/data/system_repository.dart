import '../../../core/api/api_client.dart';
import '../../../core/api/api_exception.dart';
import '../model/backend_status.dart';
import '../model/mobile_bootstrap.dart';

class SystemRepository {
  const SystemRepository(this._apiClient);

  final ApiClient _apiClient;

  Future<MobileBootstrap> fetchMobileBootstrap() async {
    final response = await _apiClient.getResponse(
      '/bootstrap/mobile',
      authenticated: false,
    );

    if (response.statusCode == 200) {
      return MobileBootstrap.fromJson(response.body);
    }

    throw ApiException(
      statusCode: response.statusCode,
      code: 'mobile_bootstrap_failed',
      message: 'Could not read mobile bootstrap metadata.',
    );
  }

  Future<BackendStatus> fetchBackendStatus() async {
    final bootstrap = await fetchMobileBootstrap();
    return bootstrap.backendStatus;
  }
}
