"""R3 to R8 — the generated service's own rules, proved by the project's own `cargo test`.

The rules themselves are `#[cfg(test)]` modules in the assets (`assets/backing-services/rust/`), written as the
service's tests and dispatched through `Router::oneshot` and `config::load_from`, so none of them opens a socket
or reads this process's environment. What this suite proves is that they *arrive* in a generated project and
that cargo runs them there: each example names the filter a rule's tests live under and a count they must reach,
so a rule whose tests were not generated cannot pass as a filter that matched nothing.
"""
from __future__ import annotations

import json
import os
import re
import subprocess
import tempfile
import unittest
from pathlib import Path

from support import FactoryTestCase, commit_all
from test_add_service import add_service

from slipwai.project.openapi import DOCUMENTS, EXPORTERS
from slipwai.project.rules import API_CONTRACTS

CARGO_ENVIRONMENT = {**os.environ, "CARGO_BUILD_JOBS": "2"}


class GeneratedServiceTest(FactoryTestCase):
    """One standard-profile Rust service on `axum`, generated once and tested per rule."""

    directory: tempfile.TemporaryDirectory[str]
    repo: Path

    @classmethod
    def setUpClass(cls) -> None:
        cls.directory = tempfile.TemporaryDirectory()
        cls.repo = cls().generate(cls.directory.name, "served", "standard", "rust", http="axum")

    @classmethod
    def tearDownClass(cls) -> None:
        cls.directory.cleanup()

    def cargo_test(self, filter_: str, at_least: int) -> None:
        """Runs the service's tests under `filter_` and requires that at least `at_least` ran and all passed."""
        result = subprocess.run(
            ["cargo", "test", "--locked", "--lib", filter_],
            cwd=self.repo / "apps/service", env=CARGO_ENVIRONMENT, text=True, capture_output=True,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        ran = re.search(r"test result: ok\. (\d+) passed", result.stdout)
        self.assertIsNotNone(ran, result.stdout)
        assert ran is not None
        self.assertGreaterEqual(int(ran.group(1)), at_least, f"only {ran.group(1)} tests ran under {filter_}")

    def cargo(self, *arguments: str) -> None:
        result = subprocess.run(
            ["cargo", *arguments], cwd=self.repo / "apps/service", env=CARGO_ENVIRONMENT, text=True, capture_output=True
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_what_has_been_generated_is_formatted_and_lint_clean(self) -> None:
        """The gate every generated project runs, over every module the transport has added so far."""
        self.cargo("fmt", "--check")
        self.cargo("clippy", "--locked", "--all-targets", "--", "-D", "warnings")

    def test_the_adapter_s_routes_are_held_by_its_own_tests(self) -> None:
        self.cargo_test("adapters::driving::http::tests", at_least=24)

    def test_what_a_browser_meets_first_is_held_by_the_wrapper_s_own_tests(self) -> None:
        self.cargo_test("adapters::driving::http::security", at_least=7)

    def test_the_checked_environment_is_held_by_its_own_tests(self) -> None:
        self.cargo_test("config::tests", at_least=11)

    def test_one_span_per_request_is_held_by_its_own_tests(self) -> None:
        self.cargo_test("observability::tests", at_least=19)

    def test_the_published_contract_is_held_to_the_router_by_its_own_test(self) -> None:
        self.cargo_test("adapters::driving::http::openapi", at_least=2)

    def test_a_route_the_document_does_not_describe_fails_that_test(self) -> None:
        """A guard has teeth only if it can be seen to bite: rename a path in the document and the test objects."""
        document = self.repo / "apps/service/openapi.yaml"
        original = document.read_text()
        try:
            document.write_text(original.replace("\n  /ready:\n", "\n  /readyz:\n"))
            result = subprocess.run(
                ["cargo", "test", "--locked", "--lib", "adapters::driving::http::openapi"],
                cwd=self.repo / "apps/service", env=CARGO_ENVIRONMENT, text=True, capture_output=True,
            )
        finally:
            document.write_text(original)
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn("GET /ready is served and openapi.yaml does not describe it", result.stdout + result.stderr)

    def test_the_document_describes_what_is_served_and_nothing_this_slice_does_not(self) -> None:
        document = (self.repo / "apps/service/openapi.yaml").read_text()

        for path in ("/health", "/ready"):
            self.assertIn(f"\n  {path}:\n", document)
        for status in ('"200"', '"503"'):
            self.assertIn(f"        {status}:\n", document)
        for schema in ("Health", "Ready", "Unready", "SchemaFailure", "NotFound"):
            self.assertIn(f"\n    {schema}:\n", document)
        # The flags route arrives with the first production target, so a document that described it would
        # promise a path nothing serves.
        self.assertNotIn("/api/flags", document)
        self.assertNotIn("Flags:", document)

    def test_the_document_is_published_by_name_and_has_no_exporter_or_recipe(self) -> None:
        self.assertEqual(DOCUMENTS["axum"], "openapi.yaml")
        self.assertIn("axum", API_CONTRACTS)
        # Hand-written, as Go's is: the router cannot list its own routes, so nothing writes the file out.
        self.assertNotIn("axum", EXPORTERS)
        self.assertNotIn("check-openapi", (self.repo / "Makefile").read_text())


class TraceCorrelationTest(FactoryTestCase):
    """What a transport brings with it elsewhere: the request's trace as an event's correlation and causation ids."""

    EXPORTS = {
        "go": ("go/tracing.go", r"^func (?:\([^)]*\) )?([A-Z]\w*)\(", {"TraceIDs": "trace_ids"}),
        "typescript": ("typescript/tracing.ts", r"^export function (\w+)\(", {"traceIds": "trace_ids"}),
        "python": ("python/tracing.py", r"^def ([a-z]\w*)\(", {"trace_ids": "trace_ids"}),
    }

    def test_every_helper_the_other_backends_export_has_a_counterpart_or_a_written_reason(self) -> None:
        from slipwai.assets import ROOT

        note = "\n".join(
            line for line in (ROOT / "assets/backing-services/rust/observability.rs").read_text().splitlines()
            if line.startswith("//!")
        )
        for language, (path, pattern, _) in self.EXPORTS.items():
            exported = re.findall(pattern, (ROOT / "assets/backing-services" / path).read_text(), re.M)
            self.assertTrue(exported, language)
            for name in exported:
                with self.subTest(language=language, helper=name):
                    self.assertIn(name, note)

    def test_the_ids_are_ones_the_event_types_accept(self) -> None:
        """Appended to a generated project's module as a test, so the types the events module has — which a
        project with no store does not — parse what `trace_ids` returns."""
        with tempfile.TemporaryDirectory() as directory:
            repo = self.generate(directory, "correlated", "event-modelling", "rust", event_store="sqlite", http="axum")
            module = repo / "apps/service/src/observability.rs"
            module.write_text(module.read_text() + """
#[cfg(test)]
mod accepted_by_the_events_module {
    use super::*;
    use crate::application::ports::events::{CausationId, CorrelationId};

    #[test]
    fn parse_what_trace_ids_returns() {
        let provider = SdkTracerProvider::builder().build();
        let _installed = tracing::subscriber::set_default(subscriber(
            "info",
            "json",
            opentelemetry::trace::TracerProvider::tracer(&provider, "test"),
            std::io::sink,
        ));
        let router = instrument(Router::new().route(
            "/seen",
            axum::routing::get(|| async {
                let ids = trace_ids().expect("inside a request");
                CorrelationId::parse(&ids.correlation).expect("a correlation id");
                CausationId::parse(&ids.causation).expect("a causation id");
                "ok"
            }),
        ));
        tokio::runtime::Builder::new_current_thread().build().expect("a runtime").block_on(async {
            use tower::ServiceExt;
            let response = router
                .oneshot(
                    axum::http::Request::builder()
                        .uri("/seen")
                        .body(axum::body::Body::empty())
                        .expect("a request"),
                )
                .await
                .expect("a response");
            assert_eq!(response.status(), 200);
        });
    }
}
""")
            result = subprocess.run(
                ["cargo", "test", "--locked", "--lib", "accepted_by_the_events_module"],
                cwd=repo / "apps/service", env=CARGO_ENVIRONMENT, text=True, capture_output=True,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertRegex(result.stdout, r"test result: ok\. 1 passed")


class GeneratedEnvironmentTest(FactoryTestCase):
    """R5: `.env.example` carries the transport's keys, and each store's in its own region — all of them read."""

    TRANSPORT_KEYS = ("PORT", "PUBLIC_BASE_URL", "CORS_ALLOWED_ORIGINS", "OTEL_EXPORTER_OTLP_ENDPOINT")

    def keys_of(self, repo: Path) -> set[str]:
        return set(re.findall(r"^([A-Z][A-Z_]+)=", (repo / ".env.example").read_text(), re.M))

    def test_each_answer_carries_its_own_keys_and_the_transport_s(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            answers = (("sqlite", "EVENT_STORE_PATH", "DATABASE_URL"), ("postgres", "DATABASE_URL", "EVENT_STORE_PATH"))
            for store, own, other in answers:
                repo = self.generate(
                    directory, f"env-{store}", "event-modelling", "rust", event_store=store, http="axum"
                )
                keys = self.keys_of(repo)
                with self.subTest(store=store):
                    self.assertTrue(set(self.TRANSPORT_KEYS) <= keys, keys)
                    self.assertIn(own, keys)
                    self.assertNotIn(other, keys)

    def test_a_service_with_a_transport_and_no_store_has_an_environment_template_too(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = self.generate(directory, "env-standard", "standard", "rust", http="axum")

            self.assertTrue(set(self.TRANSPORT_KEYS) <= self.keys_of(repo))

    def test_every_key_the_template_documents_is_one_the_loader_reads(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = self.generate(directory, "env-read", "event-modelling", "rust", event_store="postgres", http="axum")
            loader = (repo / "apps/service/src/config.rs").read_text()

            for key in self.keys_of(repo) - {"NODE_ENV", "POSTGRES_PORT"}:
                with self.subTest(key=key):
                    self.assertIn(f'"{key}"', loader)


if __name__ == "__main__":
    unittest.main()


class AddedServiceTest(FactoryTestCase):
    def test_a_record_that_never_asked_the_transport_question_gives_the_new_service_no_transport(self) -> None:
        """A Rust project recorded before it was asked the transport question has no `http` key, which reads as the
        axis's `absent` (D2), so the service added beside its first one inherits that — not the catalog default."""
        with tempfile.TemporaryDirectory() as directory:
            repo = self.generate(directory, "older", "event-modelling", "rust", event_store="sqlite", http="none")
            document = json.loads((repo / "project.json").read_text())
            for deployable in document["deployables"].values():
                deployable.get("selection", {}).pop("http", None)
            (repo / "project.json").write_text(json.dumps(document, indent=2) + "\n")
            commit_all(repo, "as recorded before the transport question")

            result = add_service(repo, "payments")
            self.assertEqual(result.returncode, 0, result.stderr)
            payments = json.loads((repo / "project.json").read_text())["deployables"]["payments"]
            self.assertEqual(payments["selection"].get("http"), "none")
            self.assertFalse((repo / "apps/payments/src/bin/serve.rs").exists())
            # A flag still wins over the inherited answer.
            commit_all(repo, "payments")
            result = add_service(repo, "billing", "--http", "axum")
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue((repo / "apps/billing/src/bin/serve.rs").is_file())
