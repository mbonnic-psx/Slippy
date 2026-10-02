//! This process's traces and logs: the SDK, wired; the exporter, only when somewhere was named to send to.
//!
//! # Why the SDK ships and the exporter does not
//!
//! A trace id is worth having before anything collects it. It is what ties a log line to the request that
//! produced it, what an incoming `traceparent` carries in from whoever called this service, and — for a
//! project that records events — what a business transaction is correlated by. None of that needs a
//! collector, and all of it needs the SDK.
//!
//! What a collector would need is an address, and there is no honest default for one. A starter pointing at
//! `http://localhost:4318` either finds nothing there and retries a connection nobody asked for, or finds
//! something and ships a project's traffic somewhere it was never told about. So the rule is the one the
//! variable states: set `OTEL_EXPORTER_OTLP_ENDPOINT` and spans are exported; leave it unset and they are
//! recorded, given ids, and dropped. The service starts and `make verify` passes with nothing listening either
//! way.
//!
//! # Built before the runtime
//!
//! The exporter uses a blocking HTTP client, which must not be built or dropped inside an async context. The
//! entry point therefore calls [`start_tracing`] from a plain `fn main`, builds the tokio runtime after it,
//! and calls [`Tracing::shutdown`] once the runtime is gone.
//!
//! # An unreachable collector is a warning, never a crash
//!
//! Exporting happens on a background batch, off the request path, and [`Tracing::shutdown`] reports a failure
//! rather than returning it. Telemetry that can take the service down with it is worse than no telemetry.
//!
//! # The same helpers the other backends export
//!
//! Go, TypeScript and Python each export these to a slice, and each has its counterpart here:
//!
//! - starting tracing (`StartTracing`, `startTracing`, `start_tracing`) is [`start_tracing`], and stopping it
//!   (`Shutdown`) is [`Tracing::shutdown`];
//! - wrapping the transport in a span per request (`Instrument`, the framework's own hook) is [`instrument`];
//! - the ids an event carries (`TraceIDs`, `traceIds`, `trace_ids`) is [`trace_ids`];
//! - the trace and span on every log line (`TracingHandler` with its `Handle`, `WithAttrs` and `WithGroup`,
//!   `traceContextMixin`, `trace_context`) has no function here, for a reason: the request span records `trace_id` and `span_id` as fields and the log layer
//!   writes the current span with every line, so a line written anywhere inside a request carries both without
//!   a wrapper around the logger.

use axum::{
    Router,
    extract::{MatchedPath, Request},
    middleware::{self, Next},
    response::Response,
};
use opentelemetry::{
    propagation::{Extractor, TextMapPropagator},
    trace::{TraceContextExt, TracerProvider as _},
};
use opentelemetry_otlp::{Protocol, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    propagation::TraceContextPropagator,
    trace::{SdkTracer, SdkTracerProvider},
};
use std::str::FromStr;
use tracing::{Instrument, Subscriber, field};
use tracing_opentelemetry::{OpenTelemetryLayer, OpenTelemetrySpanExt};
use tracing_subscriber::{
    Layer, Registry,
    filter::{LevelFilter, Targets, filter_fn},
    fmt::{self, MakeWriter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

/// What [`start_tracing`] hands back: whether anything is shipped, the tracer the request spans come from, and
/// how to stop.
pub struct Tracing {
    /// True only when an endpoint was named — the one fact a start-up line should report.
    pub exporting: bool,
    tracer: SdkTracer,
    // How the spans still batched are flushed and the exporter released: the provider's own shutdown, or — in
    // this module's own tests — a function that fails on purpose. A field rather than a call straight through
    // to the provider, because "a failure here is reported and never fatal" is a rule, and a rule nothing can
    // make fail is a rule nothing proves.
    flush: Box<dyn Fn() -> Result<(), String> + Send + Sync>,
}

/// Records spans, and exports them only where `OTEL_EXPORTER_OTLP_ENDPOINT` says to.
///
/// Called before the runtime exists (see the module's note). An empty `endpoint` is the "no exporter" state,
/// and it is not the same as a disabled SDK: spans are still created, so every id is real and every log line
/// still carries one. They are simply not kept once they end.
pub fn start_tracing(service_name: &str, endpoint: &str) -> Result<Tracing, String> {
    let resource = Resource::builder()
        .with_service_name(service_name.to_owned())
        .build();
    let mut builder = SdkTracerProvider::builder().with_resource(resource);
    // Asked once, used twice: whether an endpoint was named decides both whether a batcher is attached and what
    // a start-up line reports, and two spellings of one question are two things that can disagree.
    let exporting = !endpoint.is_empty();
    if exporting {
        let exporter = SpanExporter::builder()
            .with_http()
            .with_protocol(Protocol::HttpBinary)
            .with_endpoint(format!("{}/v1/traces", endpoint.trim_end_matches('/')))
            .build()
            .map_err(|error| error.to_string())?;
        builder = builder.with_batch_exporter(exporter);
    }
    let provider = builder.build();
    let tracer = provider.tracer(service_name.to_owned());
    Ok(Tracing {
        exporting,
        tracer,
        flush: Box::new(move || provider.shutdown().map_err(|error| error.to_string())),
    })
}

impl Tracing {
    /// Flushes what is batched and releases the exporter.
    ///
    /// A flush is a network call, and process exit is the worst moment for one to fail: a collector that has
    /// gone away would otherwise turn a clean shutdown into a non-zero exit and an orchestrator's restart loop.
    /// Reported and swallowed here rather than at the call site, so nobody has to remember to.
    pub fn shutdown(&self) {
        if let Err(error) = (self.flush)() {
            tracing::warn!(%error, "traces could not be flushed on shutdown");
        }
    }

    /// Spans recorded and dropped, with nothing to flush: what logging is installed over where tracing could
    /// not be started — a refusal to start still has to be logged in this process's one shape.
    pub fn disabled() -> Self {
        Self::with_flush(|| Ok(()))
    }

    fn with_flush(flush: impl Fn() -> Result<(), String> + Send + Sync + 'static) -> Self {
        Self {
            exporting: false,
            tracer: SdkTracerProvider::builder().build().tracer("disabled"),
            flush: Box::new(flush),
        }
    }
}

/// How this process logs, decided from the environment and nowhere else — and installed as the process's one
/// subscriber. Call it once, first, so that a refusal to start is already in this process's shape.
///
/// Structured and on from the first run: a service whose only account of itself is whatever `println!`
/// somebody reached for is a service nobody can operate. JSON by default, because that is what a log shipper
/// reads; `LOG_FORMAT=pretty` — which `make dev` sets — gives the same records in the readable form a person
/// watching a terminal wants. The spans become OpenTelemetry's through the tracer, when there is one.
pub fn init_logging(level: &str, format: &str, tracing: &Tracing) {
    subscriber(level, format, tracing.tracer.clone(), std::io::stdout).init();
}

/// The binaries in this package, each a crate of its own whose records carry its own name as their target.
/// `src/bin/serve.rs` is the process that runs; `migrate` is the one that exists where there is a store to
/// migrate. Named here, where the filter is, because a binary's `tracing::info!` is otherwise a line
/// nobody sees — a factory test holds that every `src/bin/*.rs` that logs is in this list.
const BINARIES: [&str; 2] = ["serve", "migrate"];

/// The subscriber `init_logging` installs, over a writer of its choosing so a test can read what it wrote.
///
/// `level` filters this service's own records — the library's and each of its `BINARIES`. Everything else — the
/// exporter's connection pool, the HTTP server's — is held at `warn`, or a `LOG_LEVEL=debug` run drowns in what
/// its dependencies say about themselves. An unrecognised level is `info` rather than a refusal to start: a typo in a log variable must
/// never be what stops a deployment. The filter sits on the *log* layer only, so the request span still exists
/// at `warn` — it is what gives a warning its trace id, and what the exporter ships.
pub fn subscriber<W>(
    level: &str,
    format: &str,
    tracer: SdkTracer,
    writer: W,
) -> impl Subscriber + Send + Sync
where
    W: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let level = LevelFilter::from_str(level).unwrap_or(LevelFilter::INFO);
    let library = module_path!().split("::").next().unwrap_or_default();
    let targets = std::iter::once(library).chain(BINARIES).fold(
        Targets::new().with_default(LevelFilter::WARN),
        |targets, own| targets.with_target(own, level),
    );
    let lines = filter_fn(move |metadata| {
        metadata.is_span() || targets.would_enable(metadata.target(), metadata.level())
    });
    let log: Box<dyn Layer<Registry> + Send + Sync> = if format == "pretty" {
        fmt::layer().with_writer(writer).with_filter(lines).boxed()
    } else {
        fmt::layer()
            .json()
            .with_current_span(true)
            .with_span_list(false)
            .with_writer(writer)
            .with_filter(lines)
            .boxed()
    };
    Registry::default()
        .with(log)
        .with(OpenTelemetryLayer::new(tracer))
}

/// The trace and span in scope, as the ids an event carries: a correlation id and a causation id, each written
/// as the UUID those types parse.
pub struct TraceIds {
    pub correlation: String,
    pub causation: String,
}

/// The trace and span in scope, as the ids an event carries — `None` outside a request, where there is
/// nothing to correlate by and nothing is invented.
///
/// # Why this exists
///
/// Where this project records events, the port requires a correlation id and an optional causation id on every
/// one of them — and until there was a trace to take them from, every slice had to invent both. Inventing them
/// is how a causal tree ends up with everything appearing to have caused itself. The request already has an
/// identity — the span [`instrument`] opened for it, continuing whatever `traceparent` the caller sent — so
/// that is what the events it produces are correlated by, and a trace that crossed two services correlates the
/// events on both sides of it.
///
/// # Why they are re-punctuated rather than re-encoded
///
/// A correlation id is a UUID, and a W3C trace id is the same 128 bits written without hyphens: putting them
/// back invents nothing. A span id is 64 bits, half of a UUID, so it goes in the low half with the high half
/// left zero — reversible, and the zero prefix is what says at a glance that the id came from a span rather
/// than from a random generator.
///
/// ```ignore
/// if let Some(ids) = observability::trace_ids() {
///     let correlation = CorrelationId::parse(&ids.correlation)?;
///     let causation = CausationId::parse(&ids.causation)?;
/// }
/// ```
///
/// Returned as text rather than as the events module's types, because this module is in projects that record
/// no events and has no such types to name; `parse` still checks them, because the branded types exist to be
/// checked once at the edge and this is an edge like any other.
pub fn trace_ids() -> Option<TraceIds> {
    let context = tracing::Span::current().context();
    let span = context.span();
    let span_context = span.span_context();
    if !span_context.is_valid() {
        return None;
    }
    Some(TraceIds {
        correlation: hyphenate(&span_context.trace_id().to_string()),
        causation: hyphenate(&format!("0000000000000000{}", span_context.span_id())),
    })
}

/// 32 hex characters as a UUID reads them: 8-4-4-4-12.
fn hyphenate(hexadecimal: &str) -> String {
    format!(
        "{}-{}-{}-{}-{}",
        &hexadecimal[0..8],
        &hexadecimal[8..12],
        &hexadecimal[12..16],
        &hexadecimal[16..20],
        &hexadecimal[20..32]
    )
}

/// Wraps a router so every request gets a span, continuing whatever `traceparent` the caller sent.
///
/// Applied by the entry point outside [`crate::adapters::driving::http::build_app`], for the reason the
/// security wrapper is: it is the process that is exposed, and instrumentation a test has to install proves
/// nothing about the routes. The span is named for its method and the route that matched — never the path,
/// which is where a credential in a URL would end up in every trace — and records `trace_id`, so every log
/// line written inside the request carries it.
pub fn instrument(router: Router) -> Router {
    router.layer(middleware::from_fn(request_span))
}

async fn request_span(request: Request, next: Next) -> Response {
    let parent = TraceContextPropagator::new().extract(&Headers(request.headers()));
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map_or("unmatched", MatchedPath::as_str)
        .to_owned();
    let method = request.method().to_string();
    let name = format!("{method} {route}");
    let span = tracing::info_span!(
        "request",
        otel.name = %name,
        "http.request.method" = %method,
        "http.route" = %route,
        trace_id = field::Empty,
        span_id = field::Empty,
        "http.response.status_code" = field::Empty,
    );
    // A request that carried no valid `traceparent` leaves the span the root of a fresh trace.
    let _ = span.set_parent(parent);
    let context = span.context();
    let span_context = context.span().span_context().clone();
    span.record("trace_id", field::display(span_context.trace_id()));
    span.record("span_id", field::display(span_context.span_id()));
    let response = next.run(request).instrument(span.clone()).await;
    span.record("http.response.status_code", response.status().as_u16());
    response
}

struct Headers<'a>(&'a axum::http::HeaderMap);

impl Extractor for Headers<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|value| value.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(axum::http::HeaderName::as_str).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::get;
    use opentelemetry_sdk::error::OTelSdkResult;
    use opentelemetry_sdk::trace::{SpanData, SpanExporter};
    use std::sync::{Arc, Mutex};
    use tower::ServiceExt;

    // A traceparent as W3C writes one: version, trace id, span id, flags.
    const INCOMING: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";

    /// A hand-written exporter that keeps what it was given. A fake rather than a framework: what is asserted is
    /// the spans the request produced, and a mock would only prove that `export` was called.
    #[derive(Debug, Clone, Default)]
    struct Recorder(Arc<Mutex<Vec<SpanData>>>);

    impl SpanExporter for Recorder {
        async fn export(&self, batch: Vec<SpanData>) -> OTelSdkResult {
            self.0.lock().expect("the recorder's lock").extend(batch);
            Ok(())
        }
    }

    /// Everything written to it, as the one place a log line lands in these tests.
    #[derive(Clone, Default)]
    struct Written(Arc<Mutex<Vec<u8>>>);

    impl Written {
        fn text(&self) -> String {
            String::from_utf8(self.0.lock().expect("the buffer's lock").clone()).expect("utf-8")
        }
    }

    impl std::io::Write for Written {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0
                .lock()
                .expect("the buffer's lock")
                .extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Written {
        type Writer = Written;

        fn make_writer(&'a self) -> Written {
            self.clone()
        }
    }

    /// The provider every test but the last builds: spans recorded and kept by the fake, nothing exported.
    struct Harness {
        recorder: Recorder,
        written: Written,
        provider: SdkTracerProvider,
        _installed: tracing::subscriber::DefaultGuard,
    }

    fn harness(level: &str, format: &str) -> Harness {
        let recorder = Recorder::default();
        let provider = SdkTracerProvider::builder()
            .with_simple_exporter(recorder.clone())
            .build();
        let written = Written::default();
        let installed = tracing::subscriber::set_default(subscriber(
            level,
            format,
            provider.tracer("test"),
            written.clone(),
        ));
        Harness {
            recorder,
            written,
            provider,
            _installed: installed,
        }
    }

    async fn request(router: Router, uri: &str, traceparent: Option<&str>) {
        let mut builder = Request::builder().uri(uri);
        if let Some(traceparent) = traceparent {
            builder = builder.header("traceparent", traceparent);
        }
        router
            .oneshot(builder.body(Body::empty()).expect("a request"))
            .await
            .expect("a response");
    }

    fn app() -> Router {
        instrument(Router::new().route(
            "/orders/{id}",
            get(|| async {
                tracing::info!("handled");
                "ok"
            }),
        ))
    }

    fn spans(harness: &Harness) -> Vec<SpanData> {
        harness.provider.force_flush().expect("a flush");
        harness
            .recorder
            .0
            .lock()
            .expect("the recorder's lock")
            .clone()
    }

    #[tokio::test]
    async fn a_request_with_a_traceparent_continues_the_callers_trace() {
        let harness = harness("info", "json");

        request(app(), "/orders/1", Some(INCOMING)).await;

        let spans = spans(&harness);
        assert_eq!(spans.len(), 1);
        assert_eq!(
            spans[0].span_context.trace_id().to_string(),
            "4bf92f3577b34da6a3ce929d0e0e4736"
        );
        assert_eq!(spans[0].parent_span_id.to_string(), "00f067aa0ba902b7");
    }

    #[tokio::test]
    async fn a_request_without_one_starts_a_fresh_trace() {
        let harness = harness("info", "json");

        request(app(), "/orders/1", None).await;
        request(app(), "/orders/2", None).await;

        let spans = spans(&harness);
        assert_eq!(spans.len(), 2);
        assert!(spans[0].span_context.is_valid());
        assert_ne!(
            spans[0].span_context.trace_id(),
            spans[1].span_context.trace_id()
        );
        assert_eq!(
            spans[0].parent_span_id,
            opentelemetry::trace::SpanId::INVALID
        );
    }

    #[tokio::test]
    async fn the_span_is_named_for_its_method_and_matched_route_and_never_the_path() {
        let harness = harness("info", "json");

        request(app(), "/orders/tok-live-abc123", None).await;
        request(app(), "/nowhere/tok-live-abc123", None).await;

        let names: Vec<String> = spans(&harness)
            .iter()
            .map(|span| span.name.to_string())
            .collect();
        assert_eq!(names, ["GET /orders/{id}", "GET unmatched"]);
        assert!(
            !harness.written.text().contains("tok-live-abc123"),
            "{}",
            harness.written.text()
        );
    }

    #[tokio::test]
    async fn every_line_written_during_a_request_carries_its_trace() {
        let harness = harness("info", "json");

        request(app(), "/orders/1", Some(INCOMING)).await;

        let line: serde_json::Value =
            serde_json::from_str(harness.written.text().lines().next().expect("a line"))
                .expect("json");
        assert_eq!(line["fields"]["message"], "handled");
        assert_eq!(line["span"]["trace_id"], "4bf92f3577b34da6a3ce929d0e0e4736");
    }

    /// What `trace_ids` saw from inside a handler.
    fn seen_by_a_handler(traceparent: Option<&str>) -> (Option<TraceIds>, Vec<SpanData>, Harness) {
        let harness = harness("info", "json");
        let seen = Arc::new(Mutex::new(None));
        let handler_saw = seen.clone();
        let router = instrument(Router::new().route(
            "/seen",
            get(move || {
                let handler_saw = handler_saw.clone();
                async move {
                    *handler_saw.lock().expect("the lock") = Some(trace_ids());
                    "ok"
                }
            }),
        ));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime");
        runtime.block_on(request(router, "/seen", traceparent));
        let ids = seen
            .lock()
            .expect("the lock")
            .take()
            .expect("the handler ran");
        let recorded = spans(&harness);
        (ids, recorded, harness)
    }

    #[test]
    fn nothing_invents_an_id_outside_a_request() {
        let _harness = harness("info", "json");

        assert!(trace_ids().is_none());
    }

    #[test]
    fn an_event_is_correlated_by_the_trace_the_caller_sent_in() {
        let (ids, spans, _harness) = seen_by_a_handler(Some(INCOMING));
        let ids = ids.expect("inside a request");

        // The caller's trace id, re-punctuated as the UUID a correlation id is: the same 128 bits, so a log
        // line in the other service and an event here name one transaction.
        assert_eq!(ids.correlation, "4bf92f35-77b3-4da6-a3ce-929d0e0e4736");
        // The cause is *this* service's request span and not the caller's, in the low half of a UUID.
        let span_id = spans[0].span_context.span_id().to_string();
        assert_eq!(
            ids.causation,
            format!("00000000-0000-0000-{}-{}", &span_id[..4], &span_id[4..])
        );
        assert_ne!(span_id, "00f067aa0ba902b7");
    }

    #[test]
    fn a_request_with_no_traceparent_is_still_correlated_by_its_own_fresh_trace() {
        let (ids, spans, _harness) = seen_by_a_handler(None);
        let ids = ids.expect("inside a request");

        let trace = spans[0].span_context.trace_id().to_string();
        assert_eq!(ids.correlation.replace('-', ""), trace);
        assert_eq!(ids.correlation.len(), 36);
    }

    // `CorrelationId::parse` and `CausationId::parse` take a UUID in the form `Uuid::parse_str` does, which is
    // lower-case hex as 8-4-4-4-12; a factory test holds that they accept these where the project records
    // events, because this module is in projects that do not have those types.
    #[test]
    fn both_ids_are_written_the_way_a_uuid_is() {
        let (ids, _, _harness) = seen_by_a_handler(Some(INCOMING));
        let ids = ids.expect("inside a request");

        for id in [&ids.correlation, &ids.causation] {
            let groups: Vec<usize> = id.split('-').map(str::len).collect();
            assert_eq!(groups, [8, 4, 4, 4, 12], "{id}");
            assert!(
                id.chars()
                    .all(|c| c == '-' || c.is_ascii_digit() || ('a'..='f').contains(&c)),
                "{id}"
            );
        }
    }

    #[test]
    fn every_line_written_during_a_request_carries_its_span_as_well_as_its_trace() {
        let harness = harness("info", "json");

        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime")
            .block_on(request(app(), "/orders/1", Some(INCOMING)));

        let line: serde_json::Value =
            serde_json::from_str(harness.written.text().lines().next().expect("a line"))
                .expect("json");
        let span_id = spans(&harness)[0].span_context.span_id().to_string();
        assert_eq!(line["span"]["span_id"], span_id);
    }

    #[test]
    fn a_line_written_outside_a_request_carries_no_trace() {
        let harness = harness("info", "json");

        tracing::info!("started");

        let line: serde_json::Value =
            serde_json::from_str(harness.written.text().lines().next().expect("a line"))
                .expect("json");
        assert!(line.get("span").is_none(), "{line}");
    }

    #[test]
    fn the_level_filters_this_service_and_everything_else_is_held_at_warn() {
        let quiet = harness("warn", "json");
        tracing::info!("hidden");
        tracing::warn!("shown");
        assert_eq!(quiet.written.text().lines().count(), 1);
        drop(quiet);

        let chatty = harness("debug", "json");
        tracing::debug!("this service, at debug");
        tracing::info!(target: "hyper_util", "a dependency, at info");
        tracing::warn!(target: "hyper_util", "a dependency, at warn");
        let text = chatty.written.text();
        assert!(
            text.contains("this service, at debug") && text.contains("a dependency, at warn"),
            "{text}"
        );
        assert!(!text.contains("a dependency, at info"), "{text}");
    }

    // An entry point is a crate of its own, so its records carry its own name as their target and not this
    // library's. The one that says where the service is listening is `serve`'s, and a filter that admitted
    // only this crate would drop it at every level but `warn`.
    #[test]
    fn the_package_s_own_binaries_log_at_the_asked_level_as_this_library_does() {
        let harness = harness("info", "json");

        tracing::info!(target: "serve", "service listening");
        tracing::info!(target: "migrate", "migrations applied");
        tracing::debug!(target: "serve", "below the asked level");
        tracing::info!(target: "hyper_util", "a dependency, at info");

        let text = harness.written.text();
        assert!(
            text.contains("service listening") && text.contains("migrations applied"),
            "{text}"
        );
        assert!(!text.contains("below the asked level"), "{text}");
        assert!(!text.contains("a dependency, at info"), "{text}");
    }

    #[test]
    fn an_unknown_level_is_read_as_info() {
        let harness = harness("shouting", "json");

        tracing::debug!("hidden");
        tracing::info!("shown");

        assert_eq!(harness.written.text().lines().count(), 1);
    }

    #[test]
    fn pretty_is_for_a_terminal_and_anything_else_is_json() {
        let pretty = harness("info", "pretty");
        tracing::info!("hello");
        assert!(serde_json::from_str::<serde_json::Value>(pretty.written.text().trim()).is_err());
        drop(pretty);

        let other = harness("info", "whatever");
        tracing::info!("hello");
        assert!(serde_json::from_str::<serde_json::Value>(other.written.text().trim()).is_ok());
    }

    #[test]
    fn no_endpoint_records_spans_and_exports_nothing() {
        let tracing = start_tracing("tracing-test", "").expect("tracing starts");

        assert!(!tracing.exporting);
        tracing.shutdown();
    }

    // Built and immediately shut down: what is asserted is the decision, and an exporter pointing at an address
    // nothing is listening on has to be constructible without anything failing — which is exactly the state a
    // service started with a collector that is down is in. A plain `#[test]`, because the blocking client the
    // exporter uses must not be built or dropped inside an async context.
    #[test]
    fn an_endpoint_is_what_decides_whether_anything_is_exported() {
        let tracing = start_tracing("tracing-test", "http://127.0.0.1:1")
            .expect("an exporter with nothing listening must still build");

        assert!(tracing.exporting);
        tracing.shutdown();
    }

    #[test]
    fn a_flush_that_failed_is_reported_and_never_fatal() {
        // A hand-written stand-in for the provider's own shutdown, failing the way a collector that has gone
        // away fails. Process exit is the worst moment for a network call to raise, and this is the rule that
        // says it does not — so the rule is driven rather than only described.
        let harness = harness("info", "json");
        let tracing = Tracing::with_flush(|| Err("collector gone".to_owned()));

        tracing.shutdown();

        let text = harness.written.text();
        assert!(
            text.contains("traces could not be flushed on shutdown"),
            "{text}"
        );
        assert!(text.contains("collector gone"), "{text}");
    }

    #[test]
    fn a_flush_that_worked_says_nothing() {
        let harness = harness("info", "json");

        Tracing::with_flush(|| Ok(())).shutdown();

        assert_eq!(harness.written.text(), "");
    }
}
