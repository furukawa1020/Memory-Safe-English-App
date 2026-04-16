from app.application import build_application
from app.config import Settings


def test_build_application_uses_settings() -> None:
    app = build_application(
        Settings(
            host="127.0.0.1",
            port=9000,
            max_words_per_chunk=3,
            require_api_key=True,
            api_keys=("k1",),
            require_request_signature=True,
            signature_keys=("s1",),
        )
    )

    assert app.settings.port == 9000
    assert app.chunking_service.max_words_per_chunk == 3
    assert app.rate_limiter.max_requests == app.settings.rate_limit_max_requests


def test_build_application_rejects_missing_required_api_key() -> None:
    try:
        build_application(
            Settings(
                host="127.0.0.1",
                port=9000,
                max_words_per_chunk=3,
                require_api_key=True,
                require_request_signature=False,
            )
        )
    except ValueError as exc:
        assert "WORKER_API_KEYS" in str(exc)
    else:
        raise AssertionError("expected settings validation to fail")


def test_build_application_rejects_missing_required_signature_key() -> None:
    try:
        build_application(
            Settings(
                host="127.0.0.1",
                port=9000,
                max_words_per_chunk=3,
                require_api_key=True,
                api_keys=("k1",),
                require_request_signature=True,
            )
        )
    except ValueError as exc:
        assert "WORKER_SIGNATURE_KEYS" in str(exc)
    else:
        raise AssertionError("expected settings validation to fail")
