from app.application import build_application
from app.config import Settings


def test_build_application_uses_settings() -> None:
    app = build_application(Settings(host="127.0.0.1", port=9000, max_words_per_chunk=3))

    assert app.settings.port == 9000
    assert app.chunking_service.max_words_per_chunk == 3
