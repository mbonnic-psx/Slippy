//! The ports the application is driven through: traits named for what it needs, never for what implements
//! them. `events` is the write side's, `read_models` the read side's.

pub mod events;
pub mod read_models;
