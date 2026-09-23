//! Applies this service's migrations to the database `DATABASE_URL` names — what `make migrate` runs.
//!
//! The SQL under `migrations/` is compiled into this binary, so the production image carries the schema it
//! expects rather than reading a directory it does not have. Each migration runs once, in order, recorded in
//! the ledger sqlx keeps beside the schema; a migration already applied is never run again, and one that
//! fails is rolled back. A migration is never edited once it has reached `main`: the next one changes what
//! the last one did (`make check-migrations` holds that).

use std::process::ExitCode;

use sqlx::postgres::PgPoolOptions;

/// Where the database is, or why there is no answer.
fn database_url(configured: Option<String>) -> Result<String, String> {
    configured.filter(|url| !url.trim().is_empty()).ok_or_else(|| {
        "DATABASE_URL is not set. `make migrate` exports the value from the Makefile; outside make, copy it \
         from .env.example"
            .to_owned()
    })
}

async fn migrate() -> Result<(), String> {
    let url = database_url(std::env::var("DATABASE_URL").ok())?;
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .map_err(|error| format!("connect: {error}"))?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|error| error.to_string())?;
    println!("migrate: up to date");
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    match migrate().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("migrate: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn says_how_to_set_the_database_when_nothing_names_it() {
        for missing in [None, Some(String::new()), Some("  ".to_owned())] {
            let refusal = database_url(missing).expect_err("no database named");
            assert!(refusal.contains("DATABASE_URL is not set"), "{refusal}");
        }
        assert_eq!(
            database_url(Some("postgres://app@db/app".to_owned())).as_deref(),
            Ok("postgres://app@db/app")
        );
    }
}
