//! The published contract and the routes that serve it, held together.
//!
//! `openapi.yaml` is hand-written on this backend — there is no generator, and buying one would cost the
//! dependencies this backend exists without (a macro layer over the router, a YAML parser for a test). What
//! keeps a hand-written document honest is this: a route the router serves and the document does not describe
//! fails `make verify`, in the same run that added it.
//!
//! Read out of the source rather than out of the router, because an axum `Router` does not say what it
//! holds: there is no iterator over its routes to ask. The scan below finds the one spelling the module uses
//! (`route(router, "<path>", <method>(…))`), so a route registered any other way is a route this test cannot
//! see — which is the reason the count is asserted too.

/// Every `(method, path)` a source registers, in order. Only what is written before the module's own tests.
fn registrations(source: &str) -> Vec<(String, String)> {
    let production = source
        .split("#[cfg(test)]\nmod tests")
        .next()
        .unwrap_or(source);
    let mut found = Vec::new();
    let mut rest = production;
    while let Some(at) = rest.find("route(") {
        rest = &rest[at + "route(".len()..];
        let Some(open) = rest.find('"') else { break };
        // A definition or a doc line that mentions `route(` runs on to some later string; a call does not.
        if rest[..open].contains(['{', ';']) {
            continue;
        }
        let after = &rest[open + 1..];
        let Some(close) = after.find('"') else { break };
        let path = &after[..close];
        let method: String = after[close + 1..]
            .trim_start_matches(|character: char| character == ',' || character.is_whitespace())
            .chars()
            .take_while(|character| character.is_ascii_lowercase() || *character == '_')
            .collect();
        if path.starts_with('/') && !method.is_empty() {
            found.push((method, path.to_owned()));
        }
    }
    found
}

/// The lines of `document` that describe one path: from its key to the next path or the components.
fn described<'a>(document: &'a str, path: &str) -> Option<&'a str> {
    let start = document.find(&format!("\n  {path}:\n"))? + 1;
    let block = &document[start..];
    let end = block[1..]
        .find("\n  /")
        .map(|at| at + 1)
        .or_else(|| block.find("\ncomponents:"))
        .unwrap_or(block.len());
    Some(&block[..end])
}

#[test]
fn every_route_is_in_the_published_document() {
    let root = env!("CARGO_MANIFEST_DIR");
    let source = std::fs::read_to_string(format!("{root}/src/adapters/driving/http/mod.rs"))
        .expect("the adapter's own source");
    let document =
        std::fs::read_to_string(format!("{root}/openapi.yaml")).expect("the published contract");

    let routes = registrations(&source);

    assert!(
        routes.len() >= 2,
        "found {} routes in the adapter; the scan this test reads them with has gone stale",
        routes.len()
    );
    for (method, path) in routes {
        let Some(block) = described(&document, &path) else {
            panic!(
                "{} {path} is served and openapi.yaml does not describe it",
                method.to_uppercase()
            );
        };
        assert!(
            block.contains(&format!("\n    {method}:\n")),
            "openapi.yaml describes no {method} operation, and {} {path} is served",
            method.to_uppercase()
        );
    }
}

#[test]
fn the_scan_reads_a_registration_however_it_is_wrapped() {
    let source = "route(router, \"/a\", get(h))\nroute(\n    router,\n    \"/b\",\n    post(move || x),\n)\n#[cfg(test)]\nmod tests {}\nroute(r, \"/hidden\", get(h))";

    assert_eq!(
        registrations(source),
        [
            ("get".to_owned(), "/a".to_owned()),
            ("post".to_owned(), "/b".to_owned())
        ]
    );
}
