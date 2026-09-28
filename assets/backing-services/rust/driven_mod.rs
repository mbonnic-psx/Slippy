//! The driven adapters, each implementing a port the application owns. The in-memory pair ships with every
//! answer, because it is what the contract suites run against in `make verify`.

pub mod checkpoint_store_memory;
pub mod event_store_memory;
// backing-service:sqlite:begin
pub mod checkpoint_store_sqlite;
pub mod event_store_sqlite;
// backing-service:sqlite:end
// backing-service:postgres:begin
pub mod checkpoint_store_postgres;
pub mod event_store_postgres;
// backing-service:postgres:end
