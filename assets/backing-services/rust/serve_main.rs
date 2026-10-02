//! The service's HTTP entry point.
//!
//! ```text
//! make dev
//! cd apps/service && cargo run --locked --bin serve
//! ```
//!
//! This is deliberately the one file in the service with no test (coverage already leaves `src/bin/` out).
//! Everything worth asserting about the routes is asserted against the router `build_app` returns, driven by
//! `Router::oneshot` with no socket at all — a test that needs a listening port is an integration test wearing
//! an entry point's clothes, and it proves the one thing this binary does rather than anything the router
//! decides. What is left here is composition: read the environment, build the router, bind, and shut down
//! when asked.
//!
//! Nothing here parses the environment either: `config` is one struct over it, checked before anything binds,
//! so a variable this service cannot use stops the process with the variable named rather than surfacing as a
//! 500 an hour later. `HOST`, `PORT` and `PUBLIC_BASE_URL` carry the defaults `.env.example` writes down, and
//! that struct is where they are written.

__STORE_IMPORT__

fn main() -> std::process::ExitCode {
    // The environment, checked once and before anything binds. A refusal is logged in this process's one
    // shape — which is why the two log variables are read here directly and are deliberately not among the
    // ones `config` checks — and stops the process.
    let settings = match config::load() {
        Ok(settings) => settings,
        Err(error) => {
            observability::init_logging(
                &variable("LOG_LEVEL", "info"),
                &variable("LOG_FORMAT", "json"),
                &observability::Tracing::disabled(),
            );
            tracing::error!(%error, "the environment this service was given cannot be used");
            return std::process::ExitCode::FAILURE;
        }
    };
    // Before the runtime exists, because the exporter's blocking HTTP client must not be built or dropped
    // inside an async context (see `observability`). The logging is installed with the tracer it provides,
    // so every line written while a request is being served carries the trace it happened inside.
    let tracing = match observability::start_tracing(
        &settings.otel_service_name,
        &settings.otel_exporter_otlp_endpoint,
    ) {
        Ok(tracing) => tracing,
        Err(error) => {
            observability::init_logging(
                &settings.log_level,
                &settings.log_format,
                &observability::Tracing::disabled(),
            );
            tracing::error!(%error, "tracing could not start");
            return std::process::ExitCode::FAILURE;
        }
    };
    observability::init_logging(&settings.log_level, &settings.log_format, &tracing);

    let outcome = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(Into::into)
        .and_then(|runtime| runtime.block_on(run(&settings, tracing.exporting)));
    // After the runtime is gone and the server has drained: the spans of the requests that were in flight are
    // flushed rather than dropped on the way out.
    tracing.shutdown();
    match outcome {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(%error, "the service stopped");
            std::process::ExitCode::FAILURE
        }
    }
}

async fn run(settings: &config::Config, exporting: bool) -> Result<(), Box<dyn std::error::Error>> {
__STORE_OPEN__
    // The store, where this project has one, is handed in here and nowhere else. `readiness` is what puts
    // `/ready` in front of the port, so a service whose event store has gone away stops being sent traffic; a
    // project with no store passes `None` and gets a route that answers ready with no dependency to ask.
    //
    // Wrapped twice on the way out, and both wrappers are the *process*'s rather than the router's:
    // `observability::instrument` opens a span per request, continuing whatever `traceparent` the caller sent,
    // and `security::secure` carries the headers a browser is told to enforce and the answer to whether this
    // origin may ask at all. The router a test drives is the routes and nothing else — instrumentation a test
    // has to install proves nothing — and a preflight is a question no route has an answer to.
    //
    // And sealed last, around all of it: axum tells a caller which verbs a path takes, from inside the method
    // router and after every layer, and only a wrapper around the whole router takes that back for every way a
    // slice can mount a route.
    let app = http::sealed(observability::instrument(http::security::secure(
        http::build_app(vec![http::readiness(__STORE_ARGUMENT__)]),
        settings.cors_allowed_origins.clone(),
    )));

    let listener = tokio::net::TcpListener::bind(settings.address()).await?;
    // `PUBLIC_BASE_URL` rather than the bound address, because behind a proxy or a tunnel the two differ and
    // the address worth reporting is the one somebody can open.
    tracing::info!(
        url = %settings.reported_url(),
        service = %settings.otel_service_name,
        exporting_traces = exporting,
        "service listening"
    );
    // The signal is heard once and told to two listeners: the server, which stops accepting and lets what is in
    // flight finish, and the clock that gives that a deadline. Without one a single connection that never
    // finishes — a client that sent half a request and went away — keeps the process from ever exiting.
    let (stop, stopped) = tokio::sync::watch::channel(false);
    tokio::spawn(async move {
        shutdown_signal().await;
        stop.send_replace(true);
    });
    let draining = {
        let mut stopped = stopped.clone();
        async move {
            let _ = stopped.wait_for(|stopping| *stopping).await;
        }
    };
    let deadline = {
        let mut stopped = stopped;
        async move {
            let _ = stopped.wait_for(|stopping| *stopping).await;
            tokio::time::sleep(DRAIN_TIMEOUT).await;
        }
    };
    tokio::select! {
        served = axum::serve(listener, app).with_graceful_shutdown(draining) => served?,
        () = deadline => tracing::warn!(
            seconds = DRAIN_TIMEOUT.as_secs(),
            "connections still open at the shutdown deadline were dropped"
        ),
    }
    Ok(())
}

/// How long the requests in flight are given to finish once this process is asked to stop, as the Go service's
/// `Shutdown` is given: an orchestrator's own grace period is usually longer than this, and one that is not
/// would kill the process in the middle of a flush.
const DRAIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// Resolves when this process is asked to stop: SIGTERM, which is what `docker compose down` and every
/// orchestrator send, or Ctrl-C. The server then lets the requests already in flight finish — without it a
/// demo's last request dies mid-response and reads as a bug in the slice.
async fn shutdown_signal() {
    let interrupt = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut signal) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            signal.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = interrupt => {},
        () = terminate => {},
    }
}

fn variable(name: &str, fallback: &str) -> String {
    match std::env::var(name) {
        Ok(value) if !value.is_empty() => value,
        _ => fallback.to_owned(),
    }
}
