from __future__ import annotations

import json
import logging
from typing import Any


def configure_logging() -> None:
    root = logging.getLogger()
    if root.handlers:
        return

    handler = logging.StreamHandler()
    handler.setFormatter(logging.Formatter("%(message)s"))
    root.addHandler(handler)
    root.setLevel(logging.INFO)


def audit_log(event: str, **fields: Any) -> None:
    payload = {"event": event, **fields}
    logging.getLogger("worker.audit").info(json.dumps(payload, ensure_ascii=True, sort_keys=True))
