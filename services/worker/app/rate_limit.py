from __future__ import annotations

import threading
import time
from collections import deque
from dataclasses import dataclass, field


@dataclass(slots=True)
class SlidingWindowRateLimiter:
    max_requests: int
    window_seconds: int
    _events: dict[str, deque[float]] = field(default_factory=dict)
    _lock: threading.Lock = field(default_factory=threading.Lock)

    def allow(self, key: str, now: float | None = None) -> bool:
        current = now if now is not None else time.time()
        window_start = current - self.window_seconds

        with self._lock:
            bucket = self._events.setdefault(key, deque())
            while bucket and bucket[0] < window_start:
                bucket.popleft()

            if len(bucket) >= self.max_requests:
                return False

            bucket.append(current)
            return True
