//! What a browser meets before any route does: the headers it is told to enforce on every response, and the
//! answer to whether this origin may ask at all.
//!
//! Written out rather than taken from a crate: these are four strings and one list, and the dependency that
//! would supply them would also supply thirty settings this project has not decided. `tower-http`'s CORS
//! layer is not the parity answer either — it answers a preflight with 200, where this answers 204, and varies
//! on more than `Origin`. Applied outside [`super::build_app`], by the entry point, for the reason tracing is:
//! it is the process that is exposed, and the router a test drives with `oneshot` is the routes and nothing
//! else.

use axum::{
    Router,
    extract::{Request, State},
    http::{HeaderName, HeaderValue, Method, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use std::sync::Arc;

/// The headers a browser is told to enforce on every response, whatever the route.
///
/// Nosniff is the one that matters most for a JSON API — without it a browser may decide a response is HTML
/// because of what is inside it, and runs what it finds. The rest are cheap insurance that matters the day a
/// route starts returning HTML: an error page, a hosted callback, a rendered receipt.
///
/// HSTS is deliberately absent. It is a promise about a domain that a browser then refuses to let anybody
/// take back for as long as it was given for, and a starter cannot know whether this service owns its domain
/// or shares one. Add it where the deployment is known.
pub const SECURITY_HEADERS: [(&str, &str); 4] = [
    ("x-content-type-options", "nosniff"),
    ("x-frame-options", "DENY"),
    ("referrer-policy", "no-referrer"),
    (
        "content-security-policy",
        "default-src 'none'; frame-ancestors 'none'",
    ),
];

const ALLOWED_METHODS: &str = "GET, POST, PUT, PATCH, DELETE, OPTIONS";
const PREFLIGHT_MAX_AGE: &str = "600";

/// Wraps a router in the two things a browser meets before any route does.
///
/// `allowed_origins` is `CORS_ALLOWED_ORIGINS`. Empty is the default and means no CORS headers are sent to
/// anybody, which is same-origin only: the right answer for `make dev` and `make demo`, where the browser app
/// is served from its own origin and the dev server forwards `/api` here. There is deliberately no `*`: a
/// wildcard and credentials cannot be combined at all, and a wildcard without them still hands every page on
/// the internet a reader for whatever this service answers unauthenticated. An origin this service does not
/// recognise gets a reply with no `Access-Control-Allow-Origin`, and the browser refuses it — which is the
/// enforcement, since CORS is a rule browsers apply and not one this process can apply on their behalf.
///
/// `Vary: Origin` is set whenever an `Origin` came, allowed or not, and that is not optional: without it a
/// shared cache can serve the permitted origin's response — headers and all — to a request from another.
pub fn secure(router: Router, allowed_origins: Vec<String>) -> Router {
    router.layer(middleware::from_fn_with_state(
        Arc::new(allowed_origins),
        guard,
    ))
}

async fn guard(State(allowed): State<Arc<Vec<String>>>, request: Request, next: Next) -> Response {
    let origin = request.headers().get(header::ORIGIN).cloned();
    let permitted = origin
        .as_ref()
        .and_then(|origin| origin.to_str().ok())
        .is_some_and(|origin| allowed.iter().any(|candidate| candidate == origin));
    let preflight = request.method() == Method::OPTIONS
        && request
            .headers()
            .contains_key(header::ACCESS_CONTROL_REQUEST_METHOD);
    let asked = request
        .headers()
        .get(header::ACCESS_CONTROL_REQUEST_HEADERS)
        .cloned();

    // A preflight is answered here and never reaches a route: it names a method and headers the browser is
    // *asking* about, and the routes have nothing to say about a question that is not yet a request.
    let mut response = if permitted && preflight {
        StatusCode::NO_CONTENT.into_response()
    } else {
        next.run(request).await
    };

    let headers = response.headers_mut();
    for (name, value) in SECURITY_HEADERS {
        headers.insert(
            HeaderName::from_static(name),
            HeaderValue::from_static(value),
        );
    }
    if let Some(origin) = origin {
        headers.append(header::VARY, HeaderValue::from_static("Origin"));
        if permitted {
            headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
            headers.insert(
                header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                HeaderValue::from_static("true"),
            );
            if preflight {
                headers.insert(
                    header::ACCESS_CONTROL_ALLOW_METHODS,
                    HeaderValue::from_static(ALLOWED_METHODS),
                );
                headers.insert(
                    header::ACCESS_CONTROL_ALLOW_HEADERS,
                    requested_headers(asked),
                );
                headers.insert(
                    header::ACCESS_CONTROL_MAX_AGE,
                    HeaderValue::from_static(PREFLIGHT_MAX_AGE),
                );
            }
        }
    }
    response
}

/// Echoes what the preflight asked to send, or the one header every JSON caller needs.
fn requested_headers(asked: Option<HeaderValue>) -> HeaderValue {
    asked
        .filter(|asked| !asked.is_empty())
        .unwrap_or_else(|| HeaderValue::from_static("content-type"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{HeaderMap, Method, Request};
    use axum::routing::get;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tower::ServiceExt;

    fn answering(reached: Arc<AtomicBool>) -> Router {
        Router::new().route(
            "/health",
            get(move || {
                let reached = reached.clone();
                async move {
                    reached.store(true, Ordering::SeqCst);
                    "ok"
                }
            }),
        )
    }

    async fn send(
        router: Router,
        method: Method,
        origin: Option<&str>,
        headers: &[(&str, &str)],
    ) -> (u16, HeaderMap) {
        let mut request = Request::builder().method(method).uri("/health");
        if let Some(origin) = origin {
            request = request.header("origin", origin);
        }
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        let response = router
            .oneshot(request.body(Body::empty()).expect("a request"))
            .await
            .expect("a response");
        (response.status().as_u16(), response.headers().clone())
    }

    fn secured(origins: &[&str]) -> (Router, Arc<AtomicBool>) {
        let reached = Arc::new(AtomicBool::new(false));
        (
            secure(
                answering(reached.clone()),
                origins.iter().map(|origin| (*origin).to_owned()).collect(),
            ),
            reached,
        )
    }

    #[tokio::test]
    async fn every_response_carries_the_security_headers() {
        let (router, _) = secured(&[]);

        let (_, headers) = send(router, Method::GET, None, &[]).await;

        // Nosniff is the one that matters most for a JSON API: without it a browser may decide a response is
        // HTML because of what is inside it, and runs what it finds.
        assert_eq!(headers["x-content-type-options"], "nosniff");
        assert_eq!(headers["x-frame-options"], "DENY");
        assert_eq!(headers["referrer-policy"], "no-referrer");
        assert_eq!(
            headers["content-security-policy"],
            "default-src 'none'; frame-ancestors 'none'"
        );
        // HSTS is a promise about a domain, and a starter cannot know whether it owns one.
        assert!(!headers.contains_key("strict-transport-security"));
        assert!(
            !headers.contains_key("vary"),
            "no Origin came, so nothing varies"
        );
    }

    #[tokio::test]
    async fn the_headers_are_on_the_answers_the_routes_did_not_give() {
        let (router, _) = secured(&[]);
        let request = Request::builder()
            .uri("/nowhere")
            .body(Body::empty())
            .expect("a request");

        let response = router.oneshot(request).await.expect("a response");

        assert_eq!(response.headers()["x-frame-options"], "DENY");
    }

    #[tokio::test]
    async fn a_cross_origin_request_is_permitted_nothing_until_an_origin_is_allowed() {
        let (router, _) = secured(&[]);

        let (_, headers) = send(router, Method::GET, Some("http://evil.example"), &[]).await;

        assert!(!headers.contains_key("access-control-allow-origin"));
        assert!(!headers.contains_key("access-control-allow-credentials"));
        // Still varies by origin: without it a shared cache can serve one origin's response to another.
        assert_eq!(headers["vary"], "Origin");
    }

    #[tokio::test]
    async fn exactly_the_origins_it_was_given_are_permitted() {
        let (allowed, _) = secured(&["http://localhost:5173"]);
        let (refused, _) = secured(&["http://localhost:5173"]);

        let (_, permitted) = send(allowed, Method::GET, Some("http://localhost:5173"), &[]).await;
        // A near miss is a miss: one port out is a different origin, and nothing here guesses.
        let (_, other) = send(refused, Method::GET, Some("http://localhost:5174"), &[]).await;

        assert_eq!(
            permitted["access-control-allow-origin"],
            "http://localhost:5173"
        );
        assert_eq!(permitted["access-control-allow-credentials"], "true");
        assert_eq!(permitted["vary"], "Origin");
        assert!(!other.contains_key("access-control-allow-origin"));
        assert_eq!(other["vary"], "Origin");
    }

    #[tokio::test]
    async fn a_preflight_is_answered_before_any_route_runs() {
        let (router, reached) = secured(&["http://localhost:5173"]);

        let (status, headers) = send(
            router,
            Method::OPTIONS,
            Some("http://localhost:5173"),
            &[
                ("access-control-request-method", "GET"),
                ("access-control-request-headers", "authorization"),
            ],
        )
        .await;

        assert!(
            !reached.load(Ordering::SeqCst),
            "a preflight is a question about a request, not a request"
        );
        assert_eq!(status, 204);
        assert_eq!(headers["access-control-allow-headers"], "authorization");
        assert_eq!(
            headers["access-control-allow-methods"],
            "GET, POST, PUT, PATCH, DELETE, OPTIONS"
        );
        assert_eq!(headers["access-control-max-age"], "600");
        assert_eq!(
            headers["access-control-allow-origin"],
            "http://localhost:5173"
        );
        assert_eq!(headers["x-content-type-options"], "nosniff");
    }

    #[tokio::test]
    async fn a_preflight_that_names_no_headers_is_given_the_one_every_json_caller_needs() {
        let (router, _) = secured(&["http://localhost:5173"]);

        let (status, headers) = send(
            router,
            Method::OPTIONS,
            Some("http://localhost:5173"),
            &[("access-control-request-method", "POST")],
        )
        .await;

        assert_eq!(status, 204);
        assert_eq!(headers["access-control-allow-headers"], "content-type");
    }

    #[tokio::test]
    async fn a_preflight_from_an_origin_nobody_allowed_gets_no_permission_and_no_shortcut() {
        let (router, reached) = secured(&["http://localhost:5173"]);

        let (_, headers) = send(
            router,
            Method::OPTIONS,
            Some("http://evil.example"),
            &[("access-control-request-method", "GET")],
        )
        .await;

        assert!(!headers.contains_key("access-control-allow-origin"));
        assert!(!headers.contains_key("access-control-allow-methods"));
        assert_eq!(headers["vary"], "Origin");
        let _ = reached;
    }
}
