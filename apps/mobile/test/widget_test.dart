import 'package:flutter_test/flutter_test.dart';

import 'package:memory_safe_english_mobile/features/auth/model/auth_session.dart';

void main() {
  test('stored sessions need non-empty user and tokens', () {
    const usable = AuthSession(
      userId: 'usr_123',
      email: 'user@example.com',
      displayName: 'User',
      accessToken: 'access-token',
      refreshToken: 'refresh-token',
    );
    const unusable = AuthSession(
      userId: '',
      email: 'user@example.com',
      displayName: 'User',
      accessToken: '',
      refreshToken: '',
    );

    expect(usable.isUsable, isTrue);
    expect(unusable.isUsable, isFalse);
  });
}
