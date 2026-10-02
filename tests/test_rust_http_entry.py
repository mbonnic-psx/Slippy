"""R8 — the entry point, and running it: `src/bin/serve.rs`, the one file in the service with no test.

Its rule is that it composes and nothing else, so what is asserted here is what the factory writes into it for each
event-store answer, that every answer builds and lints with no warning — generated, and after the pruner has taken a
store away — and that the thing starts, answers and stops. Everything the routes decide is the service's own tests.
"""
from __future__ import annotations

import os
import re
import signal
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request
from pathlib import Path

from support import FactoryTestCase
from test_services import selected

from slipwai.roster import add_service
from slipwai.scaffold import project_files
from slipwai.services import default_apps, services_of

CARGO_ENVIRONMENT = {**os.environ, "CARGO_BUILD_JOBS": "2"}
# What a project is generated with, by what it answered: (profile, event store).
ANSWERS = (
    ("standard", None),
    ("event-modelling", "memory"),
    ("event-modelling", "sqlite"),
    ("event-modelling", "postgres"),
)


def entry(repo: Path) -> str:
    return (repo / "apps/service/src/bin/serve.rs").read_text()


def free_port() -> int:
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        return int(probe.getsockname()[1])


class EntryPointTest(FactoryTestCase):
    def generate_answered(self, directory: str, profile: str, store: str | None) -> Path:
        answers = {"event_store": store} if store is not None else {}
        return self.generate(directory, f"entry-{store or 'none'}", profile, "rust", http="axum", **answers)

    def test_a_service_with_a_transport_has_an_entry_point_and_one_without_has_none(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            served = self.generate(directory, "served", "standard", "rust", http="axum")
            quiet = self.generate(directory, "quiet", "standard", "rust", http="none")

            self.assertTrue((served / "apps/service/src/bin/serve.rs").is_file())
            self.assertFalse((quiet / "apps/service/src/bin").exists())

    def test_each_store_fills_the_entry_point_its_own_way_and_leaves_no_placeholder(self) -> None:
        expected = {
            None: ("readiness(None)", ["StoreProbe", "InMemoryEventStore"]),
            "memory": ("readiness(Some(store))", ["SqliteEventStore", "connect_lazy"]),
            "sqlite": ("SqliteEventStore::open(", ["connect_lazy", "PostgresEventStore"]),
            "postgres": ("PgPoolOptions::new().connect_lazy(", ["SqliteEventStore"]),
        }
        with tempfile.TemporaryDirectory() as directory:
            for profile, store in ANSWERS:
                repo = self.generate_answered(directory, profile, store)
                present, absent = expected[store]
                with self.subTest(store=store):
                    source = entry(repo)
                    self.assertIn(present, source)
                    for name in absent:
                        self.assertNotIn(name, source)
                    self.assertNotIn("__STORE_", source)
                    self.assertNotIn("__", source.replace("__init__", ""), "a placeholder was left behind")

    def test_the_entry_point_seals_the_whole_stack_whatever_the_store(self) -> None:
        """`Allow` is taken back outside everything — the span, the browser wrapper, every route — or a route a
        slice mounts a way the adapter did not foresee gives it back."""
        with tempfile.TemporaryDirectory() as directory:
            for profile, store in ANSWERS:
                with self.subTest(store=store):
                    self.assertIn(
                        "http::sealed(observability::instrument(http::security::secure(",
                        entry(self.generate_answered(directory, profile, store)),
                    )

    def test_the_entry_point_takes_no_flag_wiring_of_its_own_yet(self) -> None:
        from slipwai.project.flag_route import ENTRY_WIRING

        self.assertNotIn("axum", ENTRY_WIRING)
        with tempfile.TemporaryDirectory() as directory:
            self.assertNotIn("FLAG", entry(self.generate(directory, "flags", "standard", "rust", http="axum")))

    def cargo(self, repo: Path, *arguments: str) -> None:
        result = subprocess.run(
            ["cargo", *arguments], cwd=repo / "apps/service", env=CARGO_ENVIRONMENT, text=True, capture_output=True
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def holds_the_gate(self, repo: Path) -> None:
        self.cargo(repo, "fmt", "--check")
        self.cargo(repo, "build", "--locked", "--all-targets")
        self.cargo(repo, "clippy", "--locked", "--all-targets", "--", "-D", "warnings")

    def test_every_answer_builds_formatted_and_with_no_warning(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            for profile, store in ANSWERS:
                with self.subTest(store=store):
                    self.holds_the_gate(self.generate_answered(directory, profile, store))

    def test_formatting_does_not_depend_on_how_long_the_project_is_called(self) -> None:
        """The crate is named after the project and its name opens every `use` line and some string literals, so a
        line the formatter leaves alone for one name is one it wraps, or sorts differently, for another."""
        with tempfile.TemporaryDirectory() as directory:
            for profile, store in ANSWERS:
                name = f"verify-{profile}-rust-react-vite-{store or 'none'}"
                answers = {"event_store": store} if store is not None else {}
                repo = self.generate(directory, name, profile, "rust", http="axum", **answers)
                with self.subTest(project=name):
                    self.cargo(repo, "fmt", "--check")

    def test_every_binary_that_logs_is_one_the_log_filter_admits(self) -> None:
        """A binary is a crate of its own and its records carry its name as their target, so a `tracing::` call in
        `src/bin/<name>.rs` is shown only where `observability.rs` names `<name>`. Swept over every answer, since
        which binaries exist depends on the store (`migrate` is where there is one to migrate)."""
        with tempfile.TemporaryDirectory() as directory:
            for profile, store in ANSWERS:
                repo = self.generate_answered(directory, profile, store)
                source = (repo / "apps/service/src/observability.rs").read_text()
                admitted = re.search(r"BINARIES: \[&str; \d+\] = \[(.*?)\]", source)
                self.assertIsNotNone(admitted)
                assert admitted is not None
                binaries = sorted((repo / "apps/service/src/bin").glob("*.rs"))
                self.assertTrue(binaries)
                for binary in binaries:
                    with self.subTest(store=store, binary=binary.name):
                        if "tracing::" in binary.read_text():
                            self.assertIn(f'"{binary.stem}"', admitted.group(1))

    def test_make_dev_answers_health_and_ready_and_stops_when_asked(self) -> None:
        """Proven by running, in isolation: a free port, a throwaway project, and only the process this test started
        is signalled — by the group it made, so `make`, `cargo` and the server are all and only what is stopped."""
        port = free_port()
        with tempfile.TemporaryDirectory() as directory:
            repo = self.generate(directory, "running", "standard", "rust", http="axum")
            output = Path(directory) / "dev.log"
            with output.open("w") as log:
                server = subprocess.Popen(
                    ["make", "dev"], cwd=repo, env={**CARGO_ENVIRONMENT, "PORT": str(port)},
                    stdout=log, stderr=subprocess.STDOUT, start_new_session=True,
                )
            try:
                deadline = time.monotonic() + 240
                body = None
                while time.monotonic() < deadline and body is None:
                    try:
                        with urllib.request.urlopen(f"http://127.0.0.1:{port}/health", timeout=2) as response:
                            body = response.read().decode()
                    except OSError:
                        time.sleep(1)
                self.assertEqual(body, '{"status":"ok"}')
                with urllib.request.urlopen(f"http://127.0.0.1:{port}/ready", timeout=5) as response:
                    self.assertEqual(response.read().decode(), '{"status":"ready"}')
                # A known path under the wrong verb is the 404 every other path gets, with no `Allow` to say it exists.
                wrong_verb = urllib.request.Request(f"http://127.0.0.1:{port}/health", data=b"", method="POST")
                with self.assertRaises(urllib.error.HTTPError) as refused:
                    urllib.request.urlopen(wrong_verb, timeout=5)
                self.assertEqual(refused.exception.code, 404)
                self.assertIsNone(refused.exception.headers.get("Allow"))
                # `make dev` is how somebody finds where the service is: the entry point's own line says so,
                # at the level `make dev` runs at (the default), and is not filtered out as a binary's.
                self.assertIn(f"http://localhost:{port}", output.read_text())
            finally:
                os.killpg(server.pid, signal.SIGTERM)
                server.wait(timeout=60)
            self.assertIsNotNone(server.returncode)


class SeveralServicesTest(FactoryTestCase):
    def test_two_rust_services_are_told_their_dev_servers_share_one_target_directory(self) -> None:
        first = selected("rust", http="axum")
        apps = add_service(default_apps("rust", "none", first), "payments", "rust", selected("rust", http="axum"))
        two = project_files("two", "event-modelling", "none", apps)["skills/run-the-app/SKILL.md"]
        one = project_files(
            "one", "event-modelling", "none", default_apps("rust", "none", first)
        )["skills/run-the-app/SKILL.md"]

        self.assertIn("share one `target/`", two)
        self.assertIn("one at a time", two)
        self.assertIn("`make demo`", two)
        self.assertNotIn("share one `target/`", one)

    def test_the_shared_target_sentence_counts_the_rust_services_that_serve(self) -> None:
        """R8 says two Rust services *with a transport*: one that serves beside one that does not has no second dev
        server to contend with, so it is told nothing — whichever order they are in."""
        serving = selected("rust", http="axum")
        quiet = selected("rust", http="none")
        for first, second in ((serving, quiet), (quiet, serving)):
            apps = add_service(default_apps("rust", "none", first), "payments", "rust", second)
            skill = project_files("mixed", "event-modelling", "none", apps)["skills/run-the-app/SKILL.md"]

            self.assertNotIn("share one `target/`", skill)
            self.assertNotIn("Two Rust services", skill)

    def test_a_rust_service_beside_a_go_one_gets_its_own_container_port_and_dev_target(self) -> None:
        apps = add_service(
            default_apps("rust", "none", selected("rust", http="axum")),
            "payments", "go", selected("go", http="net-http"),
        )
        files = project_files("mixed", "event-modelling", "none", apps)

        services = services_of(apps)
        self.assertEqual([service.language for service in services], ["rust", "go"])
        self.assertEqual(len({service.port for service in services}), 2)
        compose = self.settings(files["docker-compose.yml"])
        makefile = files["Makefile"]
        for service in services:
            self.assertIn(f"\n  {service.name}:\n", compose)
            self.assertIn(f"\n{service.dev_target}:", makefile)
        self.assertIn("cargo run --locked --bin serve", makefile)
        self.assertIn("go run ./cmd/serve", makefile)
