from __future__ import annotations

import hmac


def has_valid_api_key(provided_key: str, allowed_keys: tuple[str, ...]) -> bool:
    if not provided_key:
        return False
    return any(hmac.compare_digest(provided_key, candidate) for candidate in allowed_keys)
