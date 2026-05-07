import 'package:flutter_test/flutter_test.dart';

import 'package:memory_safe_english_mobile/features/auth/model/auth_session.dart';

void main() {
  test('stored sessions need non-empty user and tokens', () {
    const usable = AuthSession(
      userId: 'usr_123',
      email: 'user@example.com',
      displayName: 'User',
      authProvider: 'email',
      accessToken: 'access-token',
      refreshToken: 'refresh-token',
      nativeNotice: '',
    );
    const unusable = AuthSession(
      userId: '',
      email: 'user@example.com',
      displayName: 'User',
      authProvider: 'email',
      accessToken: '',
      refreshToken: '',
      nativeNotice: '',
    );

    expect(usable.isUsable, isTrue);
    expect(unusable.isUsable, isFalse);
  });

  test('guest sessions expose friendly labels', () {
    const guest = AuthSession(
      userId: 'usr_guest',
      email: 'guest_123@guest.memory-safe.local',
      displayName: 'Guest 1234',
      authProvider: 'guest',
      accessToken: 'access-token',
      refreshToken: 'refresh-token',
      nativeNotice: 'Guest mode is active.',
    );

    expect(guest.isGuest, isTrue);
    expect(guest.visibleEmail, 'Guest session on this device');
    expect(guest.visibleDisplayName, 'Guest 1234');
    expect(guest.guestNotice, 'Guest mode is active.');
  });
}
