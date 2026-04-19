typedef JsonMap = Map<String, dynamic>;

class ApiResponse {
  const ApiResponse({
    required this.statusCode,
    required this.body,
  });

  final int statusCode;
  final JsonMap body;

  bool get isSuccess => statusCode >= 200 && statusCode < 300;
}
