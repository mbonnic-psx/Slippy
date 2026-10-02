//! The inbound HTTP driving adapter.
//!
//! A driving adapter parses untrusted input into typed commands, calls a use case, and renders the
//! outcome. It holds no business rules and makes no authorisation decision — authorisation is decided
//! inside the use case, because a rule enforced in a route handler is a rule that a second entry point
//! will not enforce.
//!
//! Status mapping belongs here precisely because the domain speaks business vocabulary:
//!
//! - 400 — schema failure: the shape is wrong
//! - 422 — business rejection: the shape is fine, the rule says no
//! - 404 — not found, INCLUDING another tenant's resource, so existence is not leaked
//! - 409 — a conflict the caller may retry differently, which is where a version conflict from the event
//!   store surfaces: the store returns it as a value, and this is the layer that gives it a status
//!
//! "Parses untrusted input" is a type's job here and not a handler's. There is no framework compiling a
//! schema, so the request and response types ARE the schema: [`ApiJson`] refuses a body carrying a field
//! the type does not name (a type that derives `Deserialize` says `#[serde(deny_unknown_fields)]`), and the
//! contract those types make is written down in `openapi.yaml` beside the service, which a test in this
//! module's `openapi` sibling holds to the routes below.
//!
//! This module imports no port: the event store is the composition root's to open, and a project on the
//! standard profile has none. What `/ready` asks is declared here as [`ReadinessProbe`], and the entry
//! point adapts whatever store it opened to it.

use axum::{
    Json, Router,
    body::Bytes,
    extract::{FromRequest, Request},
    http::{StatusCode, header},
    middleware,
    response::{IntoResponse, Response},
    routing::{MethodRouter, get},
};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::{future::Future, pin::Pin, sync::Arc};

#[cfg(test)]
mod openapi;
pub mod security;

/// What a slice's routes are mounted with: a function over the router, so this module never grows a list of
/// the application's features. Registrars are handed to [`build_app`] from the composition root.
pub type Registrar = Box<dyn FnOnce(Router) -> Router>;

/// Whatever a probe can fail with. Logged by `/ready` and never sent.
pub type ProbeError = Box<dyn std::error::Error + Send + Sync>;

/// What a probe returns: boxed because a trait object cannot name an `impl Future`.
pub type ProbeFuture<'a> = Pin<Box<dyn Future<Output = Result<(), ProbeError>> + Send + 'a>>;

/// Just enough of a driven port for this module to ask whether it answers.
///
/// Declared here rather than taken from the event store's port, for two reasons. The port's methods return
/// `impl Future`, which makes it impossible to use as a trait object; and a project on the standard
/// profile has no store and no port to import, while this module is the same file in every project on this
/// transport. The entry point adapts the store it opened to this — the cheapest honest question the port
/// already answers, so the port is not widened to carry a probe.
pub trait ReadinessProbe: Send + Sync + 'static {
    fn check(&self) -> ProbeFuture<'_>;
}

const HEALTH: &str = r#"{"status":"ok"}"#;

/// The handler for the whole service, with the slices' routes mounted on it.
pub fn build_app(registrars: Vec<Registrar>) -> Router {
    // Liveness: this process is up and answering. Unconditional on purpose — it asks nothing of any
    // dependency, because a liveness probe that fails when a database is unreachable gets the process
    // restarted when the only thing wrong is somewhere else. What gates traffic is `/ready`, which
    // [`readiness`] registers.
    let mut router = route(Router::new(), "/health", get(health));
    for register in registrars {
        router = register(router);
    }
    // Registered last and at the root, so it answers anything no other route claimed — and a known path
    // under the wrong method, which axum would otherwise answer with an empty 405 and an `Allow` header
    // naming the verbs the path does take.
    //
    // The reply says nothing the caller did not already know: no path, no method, no hint whether the
    // route exists under a different verb. A body that reflected the URL would put whatever the URL
    // carried — a no-login link, a reset path, a signed download — into every proxy and access log.
    //
    router
        .fallback(not_found)
        .method_not_allowed_fallback(not_found)
        .layer(middleware::map_response(without_allow))
}

/// Mounts a route the way this module does, which is the one thing a slice's registrar should use in place
/// of `Router::route`.
///
/// axum attaches an `Allow` header naming the verbs a path does take to its answer for any other verb, and
/// attaches it *after* every layer has run — so it is the one thing [`build_app`]'s layer cannot take back
/// from a route mounted as a method router. Mounted as a service, the whole answer passes through the layer
/// first, and the 404 above says nothing about the path.
pub fn route(router: Router, path: &str, methods: MethodRouter) -> Router {
    router.route_service(path, methods.fallback(not_found))
}

async fn without_allow(mut response: Response) -> Response {
    response.headers_mut().remove(header::ALLOW);
    response
}

async fn health() -> Response {
    json_text(StatusCode::OK, HEALTH)
}

async fn not_found() -> Response {
    json_text(StatusCode::NOT_FOUND, r#"{"error":"notFound"}"#)
}

/// A response whose body is already JSON text, so every route reports success and failure identically.
fn json_text(status: StatusCode, body: &'static str) -> Response {
    (status, [(header::CONTENT_TYPE, "application/json")], body).into_response()
}

/// Registers `/ready`: whether this service should be sent traffic.
///
/// A different question from whether it is running. `/health` is liveness and deliberately unconditional: a
/// probe that goes red because a dependency is down gets the process killed rather than taken out of the
/// pool. This one asks the driven port the service cannot work without, so a store that has gone away is
/// reported as "do not send me traffic" instead of staying invisible until the first real request fails.
///
/// A registrar rather than a route inside [`build_app`], because the store is the composition root's to open
/// and hand over. A project with none passes `None` and gets a route that answers ready with no dependency
/// to ask, which is the truth for a project whose only driven port is the clock.
///
/// The failure is logged and not sent. A caller learns the category and no more: what is wrong with this
/// service's dependencies is not something an unauthenticated prober needs, and a connection string in a
/// driver's error message is exactly what would otherwise end up in one.
pub fn readiness(probe: Option<Arc<dyn ReadinessProbe>>) -> Registrar {
    Box::new(move |router| {
        route(
            router,
            "/ready",
            get(move || {
                let probe = probe.clone();
                async move { ready(probe).await }
            }),
        )
    })
}

async fn ready(probe: Option<Arc<dyn ReadinessProbe>>) -> Response {
    let no_store = [(header::CACHE_CONTROL, "no-store")];
    if let Some(probe) = probe
        && let Err(error) = probe.check().await
    {
        tracing::error!(%error, "the event store did not answer; reporting not ready");
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            no_store,
            Json(json!({ "status": "unready", "reason": "eventStore" })),
        )
            .into_response();
    }
    (StatusCode::OK, no_store, Json(json!({ "status": "ready" }))).into_response()
}

/// A request body read into `T`, or the one 400 this service sends for a body that does not parse.
///
/// This is the whole of "a driving adapter parses untrusted input into typed commands" on this backend: the
/// type is the schema. The handler's whole obligation is to take it as an argument,
/// `ApiJson(order): ApiJson<PlaceOrder>`, and the 400 is written before the handler runs — so there is one
/// 400 body in this service and no route builds one. What comes back names the field and the rule and never
/// the value: a validation error on a field holding a token or a password must not quote it back.
pub struct ApiJson<T>(pub T);

impl<T, S> FromRequest<S> for ApiJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = SchemaFailure;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(request, state)
            .await
            .map_err(|_| SchemaFailure::new("", "invalid request body"))?;
        parse(&bytes).map(ApiJson)
    }
}

fn parse<T: DeserializeOwned>(body: &[u8]) -> Result<T, SchemaFailure> {
    let mut reader = serde_json::Deserializer::from_slice(body);
    let parsed: T = serde_path_to_error::deserialize(&mut reader).map_err(|error| {
        let path = error.path().to_string();
        schema_failure_for(&path, error.inner())
    })?;
    // A body carrying a second JSON value after the first is not one document, and taking the first and
    // dropping the rest is the kind of silence this extractor exists to remove.
    reader
        .end()
        .map_err(|_| SchemaFailure::new("", "body must hold exactly one JSON value"))?;
    Ok(parsed)
}

/// Turns what `serde_json` refused into the field and the rule, and nothing else.
fn schema_failure_for(path: &str, error: &serde_json::Error) -> SchemaFailure {
    if error.classify() != serde_json::error::Category::Data {
        return SchemaFailure::new("", "invalid request body");
    }
    let message = error.to_string();
    // serde reports these as messages and nothing else, so what is needed is read back out of them. The
    // quoted part of the first two is the caller's own key, never their value; `invalid type` and
    // `invalid value` carry the value, and are reduced to the type that was wanted.
    // The path already ends in the unknown key — it is where the parser stopped — so it is not joined again.
    if between_backticks(&message, "unknown field ").is_some() {
        return SchemaFailure::new(path, "is not a field this route accepts");
    }
    if let Some(field) = between_backticks(&message, "missing field ") {
        return SchemaFailure::new(&join(path, field), "is required");
    }
    if let Some((_, wanted)) = message.split_once(", expected ") {
        let wanted = wanted.split(" at line ").next().unwrap_or(wanted);
        return SchemaFailure::new(path, &format!("must be {wanted}"));
    }
    SchemaFailure::new(path, "is not an accepted value")
}

fn between_backticks<'a>(message: &'a str, prefix: &str) -> Option<&'a str> {
    let rest = message.strip_prefix(prefix)?.strip_prefix('`')?;
    rest.split('`').next()
}

fn join(path: &str, field: &str) -> String {
    if path == "." {
        field.to_owned()
    } else {
        format!("{path}.{field}")
    }
}

/// The 400 body, so every route reports a bad shape the same way.
///
/// The offending value is deliberately absent: a validation error on a field holding a token or a password
/// must not quote it back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaFailure {
    field: String,
    message: String,
}

impl SchemaFailure {
    /// An empty field reads as `(root)` and an empty message as `invalid request body`.
    pub fn new(field: &str, message: &str) -> Self {
        let field = if field.is_empty() || field == "." {
            "(root)"
        } else {
            field
        };
        let message = if message.is_empty() {
            "invalid request body"
        } else {
            message
        };
        Self {
            field: field.to_owned(),
            message: message.to_owned(),
        }
    }

    pub fn body(&self) -> Value {
        json!({ "error": "schemaValidationFailed", "field": self.field, "message": self.message })
    }
}

impl IntoResponse for SchemaFailure {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(self.body())).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{HeaderMap, Method, Request};
    use axum::routing::post;
    use http_body_util::BodyExt;
    use serde::Deserialize;
    use tower::ServiceExt;

    // Edge tests: the outermost surface, exercised through the real router rather than around it.
    // `Router::oneshot` runs a real request through the real routing with no socket, so these stay in
    // `make verify` — a test that needs a listening port is an integration test wearing the wrong name.
    async fn send(
        app: Router,
        method: Method,
        target: &str,
        body: &str,
    ) -> (StatusCode, HeaderMap, String) {
        let request = Request::builder()
            .method(method)
            .uri(target)
            .header("content-type", "application/json")
            .body(Body::from(body.to_owned()))
            .expect("a request");
        let response = app.oneshot(request).await.expect("a response");
        let (parts, body) = response.into_parts();
        let bytes = body.collect().await.expect("a body").to_bytes();
        (
            parts.status,
            parts.headers,
            String::from_utf8(bytes.to_vec()).expect("utf-8"),
        )
    }

    async fn fetch(app: Router, target: &str) -> (StatusCode, HeaderMap, String) {
        send(app, Method::GET, target, "").await
    }

    /// A store with nothing in it that answers, which is what a healthy new project's log is.
    struct Answering;

    impl ReadinessProbe for Answering {
        fn check(&self) -> ProbeFuture<'_> {
            Box::pin(async { Ok(()) })
        }
    }

    /// A store that cannot answer — written here, by hand, and that is the rule rather than an accident: what
    /// matters is the two answers the probe can give and the two statuses they become.
    struct Unreachable;

    impl ReadinessProbe for Unreachable {
        fn check(&self) -> ProbeFuture<'_> {
            Box::pin(async { Err("connection refused: postgres://app:secret@db".into()) })
        }
    }

    #[tokio::test]
    async fn reports_liveness() {
        let (status, _, body) = fetch(build_app(vec![]), "/health").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, r#"{"status":"ok"}"#);
    }

    #[tokio::test]
    async fn is_ready_with_nothing_to_ask_when_the_project_has_no_store() {
        let app = build_app(vec![readiness(None)]);

        let (status, headers, body) = fetch(app.clone(), "/ready").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, r#"{"status":"ready"}"#);
        assert_eq!(headers["cache-control"], "no-store");
        // Liveness is still liveness: /health answers whatever the dependencies are doing.
        assert_eq!(fetch(app, "/health").await.0, StatusCode::OK);
    }

    #[tokio::test]
    async fn is_ready_when_the_probe_answers() {
        let app = build_app(vec![readiness(Some(Arc::new(Answering)))]);

        let (status, headers, body) = fetch(app, "/ready").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, r#"{"status":"ready"}"#);
        assert_eq!(headers["cache-control"], "no-store");
    }

    #[tokio::test]
    async fn is_not_ready_when_the_probe_fails_and_does_not_say_why() {
        let app = build_app(vec![readiness(Some(Arc::new(Unreachable)))]);

        let (status, headers, body) = fetch(app, "/ready").await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        let unready: serde_json::Value = serde_json::from_str(&body).expect("json");
        assert_eq!(
            unready,
            serde_json::json!({ "status": "unready", "reason": "eventStore" })
        );
        assert_eq!(headers["cache-control"], "no-store");
        // The driver's own message — which is where a connection string ends up — stays in the log.
        assert!(
            !body.contains("connection refused") && !body.contains("secret"),
            "{body}"
        );
    }

    // A 404 body that repeats the URL puts whatever the URL carried into every access log downstream.
    // Asserted rather than assumed, because it is the kind of regression a custom handler reintroduces.
    #[tokio::test]
    async fn does_not_echo_the_requested_path_back_on_a_404() {
        let (status, _, body) = fetch(build_app(vec![]), "/orders/tok-live-abc123").await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body, r#"{"error":"notFound"}"#);
    }

    #[tokio::test]
    async fn does_not_reveal_that_a_path_exists_under_a_different_method() {
        let (status, headers, body) = send(build_app(vec![]), Method::POST, "/health", "").await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body, r#"{"error":"notFound"}"#);
        assert!(!headers.contains_key("allow"), "{headers:?}");
    }

    #[tokio::test]
    async fn registers_the_routes_it_is_given_and_holds_none_of_its_own() {
        let orders: Registrar =
            Box::new(|router| router.route("/orders/{id}", get(|| async { "order" })));
        let (status, _, body) = fetch(build_app(vec![orders]), "/orders/order-1").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "order");
    }

    // A slice's request type. The fields it names are the fields the route accepts, which is the whole of
    // the schema on this backend.
    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct PlaceOrder {
        sku: String,
        quantity: Option<u32>,
    }

    fn orders() -> Router {
        let place: Registrar = Box::new(|router| {
            router.route(
                "/orders",
                post(|ApiJson(order): ApiJson<PlaceOrder>| async move {
                    Json(serde_json::json!({ "sku": order.sku, "quantity": order.quantity }))
                }),
            )
        });
        build_app(vec![place])
    }

    async fn place(body: &str) -> (StatusCode, String) {
        let (status, _, body) = send(orders(), Method::POST, "/orders", body).await;
        (status, body)
    }

    #[tokio::test]
    async fn reads_a_body_the_type_names() {
        let (status, body) = place(r#"{"sku":"sku-1","quantity":2}"#).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, r#"{"quantity":2,"sku":"sku-1"}"#);
    }

    // serde ignores an unknown field unless the type says not to, so `{"discuont": 10}` would reach a
    // handler as a request with no discount and the caller would be told their field worked.
    #[tokio::test]
    async fn refuses_a_field_the_type_does_not_name() {
        let (status, body) = place(r#"{"sku":"sku-1","discuont":10}"#).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let failure: serde_json::Value = serde_json::from_str(&body).expect("json");
        assert_eq!(failure["error"], "schemaValidationFailed");
        assert_eq!(failure["field"], "discuont");
        assert_eq!(failure["message"], "is not a field this route accepts");
    }

    #[tokio::test]
    async fn refuses_a_field_of_the_wrong_type_without_quoting_it() {
        let (status, body) = place(r#"{"sku":"sku-1","quantity":"tok-live-abc123"}"#).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        // The rejected input is not part of the reply: a validation error on a field holding a token must
        // not quote it.
        assert!(!body.contains("tok-live-abc123"), "{body}");
        let failure: serde_json::Value = serde_json::from_str(&body).expect("json");
        assert_eq!(failure["error"], "schemaValidationFailed");
        assert_eq!(failure["field"], "quantity");
        assert!(
            failure["message"]
                .as_str()
                .expect("a message")
                .starts_with("must be "),
            "{body}"
        );
    }

    #[tokio::test]
    async fn refuses_a_missing_field_and_names_it() {
        let (status, body) = place(r#"{"quantity":1}"#).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let failure: serde_json::Value = serde_json::from_str(&body).expect("json");
        assert_eq!(failure["field"], "sku");
        assert_eq!(failure["message"], "is required");
    }

    #[tokio::test]
    async fn refuses_a_body_that_is_not_one_document() {
        let (status, body) = place(r#"{"sku":"sku-1"}{"sku":"sku-2"}"#).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let failure: serde_json::Value = serde_json::from_str(&body).expect("json");
        assert_eq!(failure["field"], "(root)");
        assert_eq!(failure["message"], "body must hold exactly one JSON value");
    }

    #[tokio::test]
    async fn refuses_a_body_that_is_not_json_at_all_without_quoting_it() {
        let (status, body) = place("tok-live-abc123").await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(!body.contains("tok-live-abc123"), "{body}");
        let failure: serde_json::Value = serde_json::from_str(&body).expect("json");
        assert_eq!(failure["field"], "(root)");
        assert_eq!(failure["message"], "invalid request body");
    }
}
