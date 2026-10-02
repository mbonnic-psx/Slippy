//! This process's configuration: one struct over the environment, read and checked before anything binds.
//!
//! Twelve-factor says configuration comes from the environment, which is a decision about where it comes
//! from and says nothing about when it is read. Read at the point of use, a missing or misspelt value is
//! discovered by the first request that needs it — in production, by a customer, as a 500 naming something
//! the operator never set. Read here, it is discovered by the process that will not start, and the error
//! names the variable.
//!
//! The standard library and nothing else: `std::env` and `str::parse` are the whole of what parsing an
//! environment needs. It is a module rather than a block inside `src/bin/serve.rs` for one reason: the entry
//! point is deliberately the one file in the service with no test, because everything it does is
//! composition. Checking the environment is not composition — it is a rule, with values that pass and values
//! that do not — so it lives where a test can drive it, with [`load_from`] handed an environment instead of
//! the process's own.
//!
//! `LOG_LEVEL` and `LOG_FORMAT` are deliberately not checked: a typo in a log variable must never be the
//! thing that stops a deployment, and the entry point falls back to `info` and to JSON.
//! `OTEL_EXPORTER_OTLP_ENDPOINT` is checked, because it is an address: a wrong one is not a slower service,
//! it is telemetry going nowhere with nothing to say so.

use std::fmt;

/// The environment this process was given, as the service is allowed to read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Defaults to `0.0.0.0` rather than to localhost because this process runs inside a container as often
    /// as beside you, and a server bound to `127.0.0.1` in a container is reachable from nothing: the
    /// published port answers, the connection is refused, and nothing in the logs says why.
    pub host: String,
    pub port: u16,
    /// The address callers reach this service on, which is not derivable from `port` — behind a proxy or a
    /// tunnel they differ. Empty is fine and the bound port is reported instead; what is refused is a value
    /// that is not an address, because that one gets printed and pasted.
    pub public_base_url: String,
    pub log_level: String,
    pub log_format: String,
    /// What this service calls itself in a trace, defaulted to its own name: a service exporting spans as
    /// `unknown_service` is one nobody can find again.
    pub otel_service_name: String,
    /// Where to send spans, and the one thing that decides whether anything is sent at all. Empty is the
    /// default and means spans are recorded and dropped; a value that is not an address is refused here
    /// rather than surfacing as an exporter that silently never connects.
    pub otel_exporter_otlp_endpoint: String,
    /// Which browser origins may call this service cross-origin. Empty — the default — is same-origin only:
    /// no CORS headers are sent to anybody. A wildcard is deliberately not a value this accepts.
    pub cors_allowed_origins: Vec<String>,
    // backing-service:sqlite:begin
    /// Where the event log lives. A path, not a URL: SQLite is a file this process opens, with no server to
    /// address. The entry point opens the store from this and from nothing else.
    pub event_store_path: String,
    // backing-service:sqlite:end
    // backing-service:postgres:begin
    /// The event store's DSN. Empty is not refused here — a service that cannot reach its store reports
    /// `/ready` as 503 rather than failing to start — but a value that is not a Postgres URL is, because
    /// that one is a typo nothing else will ever tell anybody about.
    pub database_url: String,
    // backing-service:postgres:end
}

/// Why an environment was refused. The message names the variable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError(String);

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ConfigError {}

/// Reads and checks this process's own environment.
pub fn load() -> Result<Config, ConfigError> {
    load_from(|name| std::env::var(name).ok())
}

/// Reads and checks an environment, or refuses it with the variable named.
pub fn load_from(lookup: impl Fn(&str) -> Option<String>) -> Result<Config, ConfigError> {
    // An empty variable is an unset one: a Compose file or a task definition that declares a variable it has
    // no value for hands the process an empty string, which is not an answer — it is the absence of one.
    let value = |name: &str, fallback: &str| match lookup(name) {
        Some(found) if !found.is_empty() => found,
        _ => fallback.to_owned(),
    };

    let raw_port = value("PORT", "3000");
    let port = match raw_port.parse::<u16>() {
        Ok(port) if port >= 1 => port,
        _ => {
            return Err(refusal(format!(
                "PORT must be a port number between 1 and 65535, not {raw_port:?}"
            )));
        }
    };
    let public_base_url = value("PUBLIC_BASE_URL", "");
    require_scheme(
        "PUBLIC_BASE_URL",
        &public_base_url,
        &["http://", "https://"],
    )?;
    let otel_exporter_otlp_endpoint = value("OTEL_EXPORTER_OTLP_ENDPOINT", "");
    require_scheme(
        "OTEL_EXPORTER_OTLP_ENDPOINT",
        &otel_exporter_otlp_endpoint,
        &["http://", "https://"],
    )?;
    // backing-service:postgres:begin
    let database_url = value("DATABASE_URL", "");
    require_scheme(
        "DATABASE_URL",
        &database_url,
        &["postgres://", "postgresql://"],
    )?;
    // backing-service:postgres:end

    Ok(Config {
        host: value("HOST", "0.0.0.0"),
        port,
        public_base_url,
        log_level: value("LOG_LEVEL", "info"),
        log_format: value("LOG_FORMAT", "json"),
        otel_service_name: value("OTEL_SERVICE_NAME", "delivery-starter"),
        otel_exporter_otlp_endpoint,
        cors_allowed_origins: split_origins(&value("CORS_ALLOWED_ORIGINS", "")),
        // backing-service:sqlite:begin
        event_store_path: value("EVENT_STORE_PATH", "./events.sqlite3"),
        // backing-service:sqlite:end
        // backing-service:postgres:begin
        database_url,
        // backing-service:postgres:end
    })
}

fn refusal(message: String) -> ConfigError {
    ConfigError(message)
}

/// Empty passes — the variable was not given — and anything else must begin the way an address does.
fn require_scheme(name: &str, value: &str, schemes: &[&str]) -> Result<(), ConfigError> {
    if value.is_empty() || schemes.iter().any(|scheme| value.starts_with(scheme)) {
        return Ok(());
    }
    Err(refusal(format!(
        "{name} must begin with {}, not {value:?}",
        schemes.join(" or ")
    )))
}

/// Reads a comma-separated allow-list the way a deployment writes one.
///
/// Whitespace around a comma is dropped and an empty entry is not an origin — a trailing comma in a
/// deployment's environment must not become a permission for the empty string, which is what an `Origin`
/// header carries when a request has none.
fn split_origins(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .map(str::to_owned)
        .collect()
}

impl Config {
    /// Where the server binds. An IPv6 host has to be bracketed, and nothing else in the service would notice
    /// it was not.
    pub fn address(&self) -> String {
        if self.host.contains(':') {
            format!("[{}]:{}", self.host, self.port)
        } else {
            format!("{}:{}", self.host, self.port)
        }
    }

    /// The address worth logging: the one somebody can open.
    pub fn reported_url(&self) -> String {
        if self.public_base_url.is_empty() {
            format!("http://localhost:{}", self.port)
        } else {
            self.public_base_url.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// A hand-written stand-in for the process environment: the values this run was given, and nothing else.
    /// A fake rather than a framework, and a map rather than `std::env::set_var`, so a case can describe an
    /// environment without the process having one.
    fn environment(values: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> + use<> {
        let values: HashMap<String, String> = values
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect();
        move |name| values.get(name).cloned()
    }

    fn loaded(values: &[(&str, &str)]) -> Config {
        load_from(environment(values)).expect("an environment this service can use")
    }

    fn refused(values: &[(&str, &str)]) -> String {
        load_from(environment(values))
            .expect_err("an environment this service cannot use")
            .to_string()
    }

    #[test]
    fn carries_the_defaults_the_environment_template_writes_down() {
        let config = loaded(&[]);

        assert_eq!((config.host.as_str(), config.port), ("0.0.0.0", 3000));
        assert_eq!(config.address(), "0.0.0.0:3000");
        // Nothing to report but the port it bound, so that is what it reports.
        assert_eq!(config.reported_url(), "http://localhost:3000");
        assert_eq!(
            (config.log_level.as_str(), config.log_format.as_str()),
            ("info", "json")
        );
        assert_eq!(config.otel_service_name, "delivery-starter");
        assert_eq!(config.otel_exporter_otlp_endpoint, "");
        assert!(config.cors_allowed_origins.is_empty());
    }

    // The point of checking the environment at start-up rather than at the point of use: the process that
    // cannot work refuses to start, and says which variable is why.
    #[test]
    fn refuses_a_port_it_cannot_use_and_names_it() {
        for port in ["the-usual-one", "0", "70000", "-1"] {
            let said = refused(&[("PORT", port)]);

            assert!(
                said.contains("PORT"),
                "PORT={port} was refused without naming the variable: {said}"
            );
        }
    }

    // The valid range is 1-65535 inclusive at both ends, and the cases above only exercise values outside it.
    #[test]
    fn accepts_a_port_at_either_end_of_the_valid_range() {
        assert_eq!(loaded(&[("PORT", "1")]).port, 1);
        assert_eq!(loaded(&[("PORT", "65535")]).port, 65535);
    }

    #[test]
    fn refuses_an_address_that_is_not_one() {
        assert!(
            refused(&[("PUBLIC_BASE_URL", "service.internal:3000")]).contains("PUBLIC_BASE_URL")
        );
        assert!(refused(&[("PUBLIC_BASE_URL", "ftp://service")]).contains("PUBLIC_BASE_URL"));
        assert_eq!(
            loaded(&[("PUBLIC_BASE_URL", "http://service")]).public_base_url,
            "http://service"
        );
    }

    #[test]
    fn reports_the_address_somebody_can_open() {
        // Behind a proxy or a tunnel the bound port and the public address differ, and the one worth logging
        // is the one that can be opened.
        let config = loaded(&[
            ("PORT", "8080"),
            ("PUBLIC_BASE_URL", "https://service.example.com"),
        ]);

        assert_eq!(config.reported_url(), "https://service.example.com");
        assert_eq!(config.address(), "0.0.0.0:8080");
    }

    #[test]
    fn brackets_an_ipv6_host_in_the_address_it_binds() {
        assert_eq!(
            loaded(&[("HOST", "::1"), ("PORT", "8080")]).address(),
            "[::1]:8080"
        );
    }

    #[test]
    fn an_empty_variable_is_an_unset_one() {
        // A Compose file or a task definition that declares a variable it has no value for hands the process
        // an empty string, which is not an answer — it is the absence of one.
        let config = loaded(&[("HOST", ""), ("PORT", "")]);

        assert_eq!((config.host.as_str(), config.port), ("0.0.0.0", 3000));
    }

    #[test]
    fn an_allow_list_is_read_the_way_a_deployment_writes_one() {
        let config = loaded(&[(
            "CORS_ALLOWED_ORIGINS",
            "http://a.example, http://b.example,",
        )]);

        // A trailing comma must not become a permission for the empty string, which is what an Origin header
        // carries when a request has none.
        assert_eq!(
            config.cors_allowed_origins,
            ["http://a.example", "http://b.example"]
        );
    }

    #[test]
    fn an_exporter_endpoint_that_is_not_an_address_stops_the_process() {
        // A wrong endpoint is not a slower service: it is telemetry going nowhere with nothing to say so.
        assert!(
            refused(&[("OTEL_EXPORTER_OTLP_ENDPOINT", "collector:4318")])
                .contains("OTEL_EXPORTER_OTLP_ENDPOINT")
        );
        let config = loaded(&[("OTEL_EXPORTER_OTLP_ENDPOINT", "http://collector:4318")]);
        assert_eq!(config.otel_exporter_otlp_endpoint, "http://collector:4318");
    }

    #[test]
    fn the_log_variables_are_read_and_never_refused() {
        // A typo in a log variable must never be the thing that stops a deployment.
        let config = loaded(&[("LOG_LEVEL", "shouting"), ("LOG_FORMAT", "pretty")]);

        assert_eq!(
            (config.log_level.as_str(), config.log_format.as_str()),
            ("shouting", "pretty")
        );
    }

    #[test]
    fn the_service_names_itself_in_a_trace_unless_told_otherwise() {
        assert_eq!(
            loaded(&[("OTEL_SERVICE_NAME", "orders")]).otel_service_name,
            "orders"
        );
    }
    // backing-service:sqlite:begin

    #[test]
    fn the_sqlite_store_has_a_path_and_a_default_one() {
        assert_eq!(loaded(&[]).event_store_path, "./events.sqlite3");
        assert_eq!(
            loaded(&[("EVENT_STORE_PATH", "/data/events.db")]).event_store_path,
            "/data/events.db"
        );
    }
    // backing-service:sqlite:end
    // backing-service:postgres:begin

    #[test]
    fn the_postgres_store_url_is_refused_unless_it_is_one() {
        // Empty is not refused — a service that cannot reach its store reports /ready as 503 rather than
        // failing to start — but a value that is not a Postgres URL is a typo nothing else will tell anybody.
        assert_eq!(loaded(&[]).database_url, "");
        assert!(refused(&[("DATABASE_URL", "mysql://db")]).contains("DATABASE_URL"));
        assert_eq!(
            loaded(&[("DATABASE_URL", "postgres://db/app")]).database_url,
            "postgres://db/app"
        );
        assert_eq!(
            loaded(&[("DATABASE_URL", "postgresql://db/app")]).database_url,
            "postgresql://db/app"
        );
    }
    // backing-service:postgres:end
}
