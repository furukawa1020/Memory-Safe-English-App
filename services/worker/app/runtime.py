from __future__ import annotations

from http.server import ThreadingHTTPServer

from app.application import build_application
from app.config import Settings
from app.http.handlers import create_request_handler


def create_server(settings: Settings | None = None) -> ThreadingHTTPServer:
    app = build_application(settings)
    handler = create_request_handler(app)
    return ThreadingHTTPServer((app.settings.host, app.settings.port), handler)


def run(settings: Settings | None = None) -> None:
    server = create_server(settings)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
