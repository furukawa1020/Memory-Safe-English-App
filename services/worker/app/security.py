from __future__ import annotations

import hashlib
import hmac
import time
from typing import Iterable


def has_valid_api_key(provided_key: str, allowed_keys: tuple[str, ...]) -> bool:
    if not provided_key:
        return False
    return any(hmac.compare_digest(provided_key, candidate) for candidate in allowed_keys)


def has_valid_signature(
    body: bytes,
    provided_signature: str,
    timestamp: str,
    allowed_keys: Iterable[str],
    max_age_seconds: int,
    now: int | None = None,
) -> bool:
    if not provided_signature or not timestamp:
        return False

    try:
        timestamp_value = int(timestamp)
    except ValueError:
        return False

    current = now if now is not None else int(time.time())
    if abs(current - timestamp_value) > max_age_seconds:
        return False

    message = timestamp.encode("utf-8") + b"." + body
    for key in allowed_keys:
        expected = hmac.new(key.encode("utf-8"), message, hashlib.sha256).hexdigest()
        if hmac.compare_digest(provided_signature, expected):
            return True
    return False
