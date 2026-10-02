"""The entry point's two deadlines, proven by running the built binary: a drain that ends, a header that arrives.

Isolated as `test_rust_http_entry` is: a free port, a throwaway project, and only the process this test started is
signalled. A client that opens a connection and sends part of a request line is the one that holds a server open —
for ever, where nothing bounds it.
"""
from __future__ import annotations

import os
import signal
import socket
import subprocess
import tempfile
import time
from pathlib import Path

from support import FactoryTestCase

CARGO_ENVIRONMENT = {**os.environ, "CARGO_BUILD_JOBS": "2"}
# Go's `Shutdown` is bounded at 10s and its `ReadHeaderTimeout` is 5s; a margin is added for a loaded machine.
DRAIN_SECONDS = 10
HEADER_SECONDS = 5
MARGIN = 6


def free_port() -> int:
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        return int(probe.getsockname()[1])


class RunningServiceTest(FactoryTestCase):
    def start(self, directory: str) -> tuple[subprocess.Popen[bytes], int]:
        repo = self.generate(directory, "deadlines", "standard", "rust", http="axum")
        service = repo / "apps/service"
        target = Path(directory) / "target"
        env = {k: v for k, v in CARGO_ENVIRONMENT.items() if k != "PORT"}
        build = subprocess.run(
            ["cargo", "build", "--locked", "--bin", "serve"], cwd=service, text=True, capture_output=True,
            env={**env, "CARGO_TARGET_DIR": str(target)},
        )
        self.assertEqual(build.returncode, 0, build.stdout + build.stderr)
        port = free_port()
        with (Path(directory) / "serve.log").open("w") as log:
            server = subprocess.Popen(
                [str(target / "debug/serve")], cwd=service, env={**env, "PORT": str(port)},
                stdout=log, stderr=subprocess.STDOUT, start_new_session=True,
            )

        def reap() -> None:
            if server.poll() is None:
                os.killpg(server.pid, signal.SIGKILL)

        self.addCleanup(reap)
        deadline = time.monotonic() + 30
        while time.monotonic() < deadline:
            try:
                socket.create_connection(("127.0.0.1", port), timeout=1).close()
                return server, port
            except OSError:
                time.sleep(0.2)
        self.fail("the service never listened")

    def test_a_connection_that_sent_part_of_a_request_does_not_stop_the_process_stopping(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            server, port = self.start(directory)
            with socket.create_connection(("127.0.0.1", port), timeout=2) as stalled:
                stalled.sendall(b"GET /health HT")
                os.killpg(server.pid, signal.SIGTERM)
                started = time.monotonic()
                try:
                    server.wait(timeout=DRAIN_SECONDS + MARGIN)
                except subprocess.TimeoutExpired:
                    self.fail(f"the process was still running {DRAIN_SECONDS + MARGIN}s after SIGTERM")
                self.assertLess(time.monotonic() - started, DRAIN_SECONDS + MARGIN)

    def test_a_connection_that_never_finishes_its_headers_is_closed_by_the_server(self) -> None:
        """Slowloris: part of a request line, then silence. Closed after the header-read timeout, with the process
        still serving — and a request sent whole, on a connection beside it, is still answered."""
        with tempfile.TemporaryDirectory() as directory:
            server, port = self.start(directory)
            with socket.create_connection(("127.0.0.1", port), timeout=2) as stalled:
                stalled.sendall(b"GET /health HT")
                stalled.settimeout(HEADER_SECONDS + MARGIN)
                started = time.monotonic()
                try:
                    closed = stalled.recv(1024)
                except TimeoutError:
                    self.fail(f"the connection was still open {HEADER_SECONDS + MARGIN}s after a partial request")
                self.assertLess(time.monotonic() - started, HEADER_SECONDS + MARGIN)
                self.assertNotIn(b"200", closed)
            with socket.create_connection(("127.0.0.1", port), timeout=2) as whole:
                whole.sendall(b"GET /health HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n")
                self.assertIn(b'{"status":"ok"}', whole.makefile("rb").read())
            self.assertIsNone(server.poll())
