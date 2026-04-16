from __future__ import annotations

import socket
from http.server import ThreadingHTTPServer

from app.application import build_application
from app.config import Settings
from app.http.handlers import create_request_handler


class WorkerHTTPServer(ThreadingHTTPServer):
    daemon_threads = True
    allow_reuse_address = True

    def __init__(self, server_address: tuple[str, int], request_handler_class: type, timeout_seconds: int) -> None:
        super().__init__(server_address, request_handler_class)
        self._timeout_seconds = timeout_seconds

    def get_request(self) -> tuple[socket.socket, tuple[str, int]]:
        request, client_address = super().get_request()
        request.settimeout(self._timeout_seconds)
        return request, client_address


def create_server(settings: Settings | None = None) -> ThreadingHTTPServer:
    app = build_application(settings)
    handler = create_request_handler(app)
    return WorkerHTTPServer((app.settings.host, app.settings.port), handler, app.settings.request_timeout_seconds)


def run(settings: Settings | None = None) -> None:
    server = create_server(settings)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
