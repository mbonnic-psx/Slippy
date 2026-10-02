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
        .layer(middleware::map_response(without_hints))
}

/// What [`sealed`] does to every answer: `Allow` goes, and so does a bare 405, which says the same thing in a
/// status — that the path exists under another verb. A route mounted as a service (`route_service`,
/// `nest_service`) is answered by its own method router, which never reaches the router-level fallback
/// [`build_app`] sets, so the 404 the contract promises is written here for it.
async fn without_hints(response: Response) -> Response {
    if response.status() == StatusCode::METHOD_NOT_ALLOWED {
        return not_found().await;
    }
    without_allow(response).await
}

/// Mounts a route so that [`build_app`] alone answers a wrong verb with the 404 and no `Allow` — which a
/// registrar may use, but does not have to: the entry point seals the finished stack with [`sealed`], and that
/// holds for a plain `Router::route`, `nest` and `merge` too.
///
/// axum attaches an `Allow` header naming the verbs a path does take to its answer for any other verb, and
/// attaches it *after* every layer has run — so it is the one thing [`build_app`]'s layer cannot take back
/// from a route mounted as a method router. Mounted as a service, the whole answer passes through the layer
/// first, and the 404 above says nothing about the path.
pub fn route(router: Router, path: &str, methods: MethodRouter) -> Router {
    router.route_service(path, methods.fallback(not_found))
}

/// The finished stack as one service, with every answer it gives stripped of `Allow` — whichever way a
/// registrar mounted the route that gave it.
///
/// axum adds `Allow` to its answer for a verb a path does not take, inside the method router and after every
/// layer around that route has run. [`build_app`] can take it back from routes mounted through [`route`], and
/// from nothing else; a layer on the [`Router`] sits inside the route, so a plain `Router::route`, a nested
/// router or a merged one would each bring it back. What can take it back from all of them is a layer outside
/// the router as a whole, which is what this is: the router, as the one fallback of an outer router that has
/// no routes of its own to add the header to, with the strip layered around that. The entry point applies
/// it last, outside the request span and the security wrapper, so the guarantee holds by construction and not
/// by the next slice remembering which helper to mount with.
pub fn sealed(router: Router) -> Router {
    Router::new()
        .fallback_service(router)
        .layer(middleware::map_response(without_allow))
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
    // serde reports these as messages and nothing else, so what is needed is read back out of them — and a
    // message is read only for what does not depend on the input.
    //
    // The path already ends in the unknown key — it is where the parser stopped — so it is not joined again.
    if between_backticks(&message, "unknown field ").is_some() {
        return SchemaFailure::new(path, "is not a field this route accepts");
    }
    if let Some(field) = between_backticks(&message, "missing field ") {
        return SchemaFailure::new(&join(path, field), "is required");
    }
    if let Some(wanted) = wanted_by(&message) {
        return SchemaFailure::new(path, &format!("must be {wanted}"));
    }
    SchemaFailure::new(path, "is not an accepted value")
}

/// What the type asked for, from the four of serde's own messages that say so: `invalid type`, `invalid
/// value`, `invalid length` and `unknown variant`.
///
/// Every one of them is "<what was found>, expected <what was wanted>", and what was found is the caller's:
/// a string, a variant name, written into the message in whatever words the caller chose — including
/// `, expected `. So the wanted side is the one after the *last* delimiter, which the caller's text precedes and
/// so cannot reach, with the position serde appends (` at line N column M`) cut off the end first for the same
/// reason. Any other message — `Error::custom`, whose words are the type's author's, and may hold the input
/// anywhere — is not read at all.
fn wanted_by(message: &str) -> Option<String> {
    const SAYS_WHAT_WAS_WANTED: [&str; 4] = [
        "invalid type: ",
        "invalid value: ",
        "invalid length ",
        "unknown variant ",
    ];
    if !SAYS_WHAT_WAS_WANTED
        .iter()
        .any(|prefix| message.starts_with(prefix))
    {
        return None;
    }
    let without_position = message
        .rsplit_once(" at line ")
        .map_or(message, |(before, _)| before);
    let (_, wanted) = without_position.rsplit_once(", expected ")?;
    Some(wanted.to_owned())
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

    // What the 400 says is read out of serde's message, and serde's message quotes the caller's input in more
    // than one shape. One case per shape that carries it, each with a value holding the delimiters the parser
    // splits on — so a rule read from the wrong side of a delimiter, or from a message the type's author
    // wrote, hands the token back.
    const TOKEN: &str = "SECRET-TOKEN";

    fn refusal<T: DeserializeOwned + std::fmt::Debug>(body: &str) -> Value {
        let failure = parse::<T>(body.as_bytes()).expect_err("a refusal");
        let body = failure.body().to_string();
        assert!(!body.contains(TOKEN), "{body}");
        failure.body()
    }

    // These types are only ever refused, so nothing reads their fields.
    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct Quantity {
        quantity: u32,
    }

    // These types are only ever refused, so nothing reads their fields.
    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct Initial {
        initial: char,
    }

    // These types are only ever refused, so nothing reads their fields.
    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    enum Colour {
        Red,
        Green,
    }

    // These types are only ever refused, so nothing reads their fields.
    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct Paint {
        colour: Colour,
    }

    // These types are only ever refused, so nothing reads their fields.
    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct Pair {
        pair: (String, String),
    }

    // These types are only ever refused, so nothing reads their fields.
    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    #[serde(deny_unknown_fields)]
    struct Both {
        a: String,
        b: String,
    }

    fn custom<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Err(serde::de::Error::custom(format!("{raw}, expected a word")))
    }

    // These types are only ever refused, so nothing reads their fields.
    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct Word {
        #[serde(deserialize_with = "custom")]
        word: String,
    }

    #[test]
    fn an_invalid_type_does_not_quote_a_value_that_holds_the_delimiter() {
        let failure = refusal::<Quantity>(r#"{"quantity":"x, expected SECRET-TOKEN"}"#);

        assert_eq!(failure["field"], "quantity");
        assert_eq!(failure["message"], "must be u32");
    }

    #[test]
    fn an_invalid_type_does_not_quote_a_value_that_holds_the_position_suffix() {
        let failure =
            refusal::<Quantity>(r#"{"quantity":"SECRET-TOKEN at line 1 column 1, expected u8"}"#);

        assert_eq!(failure["message"], "must be u32");
    }

    #[test]
    fn an_invalid_value_does_not_quote_a_value_that_holds_the_delimiter() {
        let failure = refusal::<Initial>(r#"{"initial":"SECRET-TOKEN, expected a character"}"#);

        assert_eq!(failure["field"], "initial");
        assert_eq!(failure["message"], "must be a character");
    }

    #[test]
    fn an_invalid_length_does_not_quote_the_elements() {
        let failure = refusal::<Pair>(r#"{"pair":["SECRET-TOKEN`, expected x"]}"#);

        assert_eq!(failure["field"], "pair");
        assert_eq!(failure["message"], "must be a tuple of size 2");
    }

    #[test]
    fn an_unknown_variant_does_not_quote_the_variant() {
        let failure = refusal::<Paint>(r#"{"colour":"SECRET-TOKEN`, expected `Red`"}"#);

        assert_eq!(failure["field"], "colour");
        assert_eq!(failure["message"], "must be `Red` or `Green`");
    }

    #[test]
    fn a_message_the_type_wrote_is_not_read_for_a_rule() {
        let failure = refusal::<Word>(r#"{"word":"SECRET-TOKEN"}"#);

        assert_eq!(failure["field"], "word");
        assert_eq!(failure["message"], "is not an accepted value");
    }

    #[test]
    fn an_unknown_field_is_named_by_the_path_and_not_by_the_message() {
        let failure = refusal::<Both>(r#"{"a":"x","b":"y","c`, expected x":1}"#);

        // The key is the caller's own, and the path is where it is named; what the message adds is nothing.
        assert_eq!(failure["message"], "is not a field this route accepts");
    }

    #[test]
    fn a_missing_field_is_named_whatever_the_other_values_hold() {
        let failure = refusal::<Both>(r#"{"a":"` at line 1, expected SECRET-TOKEN"}"#);

        assert_eq!(failure["field"], "b");
        assert_eq!(failure["message"], "is required");
    }

    // The stack the entry point builds, around whatever a slice's registrar mounted: the routes, the browser
    // wrapper, the request span, and the seal over all of it.
    fn stack(register: impl FnOnce(Router) -> Router + 'static) -> Router {
        let registrar: Registrar = Box::new(register);
        sealed(crate::observability::instrument(security::secure(
            build_app(vec![registrar]),
            vec!["https://app.example".to_owned()],
        )))
    }

    async fn created() -> &'static str {
        "created"
    }

    // axum attaches `Allow` to its answer for a verb a path does not take, after every layer around the route
    // has run — so a layer on the router cannot take it back, and which form a registrar mounted the route
    // with decides whether it was there. The guarantee is therefore asked of every form.
    type Mounting = (
        &'static str,
        &'static str,
        Box<dyn FnOnce(Router) -> Router>,
    );

    fn mountings() -> Vec<Mounting> {
        vec![
            (
                "route",
                "/orders",
                Box::new(|r| r.route("/orders", post(created))),
            ),
            (
                "route_service",
                "/orders",
                Box::new(|r| r.route_service("/orders", post(created))),
            ),
            (
                "nest",
                "/v1/orders",
                Box::new(|r| r.nest("/v1", Router::new().route("/orders", post(created)))),
            ),
            (
                "nest_service",
                "/v1/orders",
                Box::new(|r| r.nest_service("/v1", Router::new().route("/orders", post(created)))),
            ),
            (
                "merge",
                "/orders",
                Box::new(|r| r.merge(Router::new().route("/orders", post(created)))),
            ),
            (
                "this module's route",
                "/orders",
                Box::new(|r| route(r, "/orders", post(created))),
            ),
        ]
    }

    #[tokio::test]
    async fn no_way_of_mounting_a_route_tells_the_caller_which_verbs_it_takes() {
        for (how, path, register) in mountings() {
            let (status, headers, body) = fetch(stack(register), path).await;

            assert_eq!(status, StatusCode::NOT_FOUND, "{how}");
            assert_eq!(body, r#"{"error":"notFound"}"#, "{how}");
            assert!(!headers.contains_key("allow"), "{how}: {headers:?}");
        }
    }

    #[tokio::test]
    async fn a_preflight_to_a_mounted_route_does_not_list_its_verbs_either() {
        for (how, path, register) in mountings() {
            let request = Request::builder()
                .method(Method::OPTIONS)
                .uri(path)
                .header("origin", "https://app.example")
                .header("access-control-request-method", "POST")
                .body(Body::empty())
                .expect("a request");
            let response = stack(register).oneshot(request).await.expect("a response");

            assert!(!response.headers().contains_key("allow"), "{how}");
        }
    }

    #[tokio::test]
    async fn the_verb_a_route_does_take_still_answers() {
        for (how, path, register) in mountings() {
            let (status, headers, body) = send(stack(register), Method::POST, path, "").await;

            assert_eq!(status, StatusCode::OK, "{how}");
            assert_eq!(body, "created", "{how}");
            assert!(!headers.contains_key("allow"), "{how}");
        }
    }
}
