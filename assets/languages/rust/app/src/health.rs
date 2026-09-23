//! What this service answers when asked whether it is alive.

/// The body of a health check: `{"status":"ok"}` once it is serialised.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub status: &'static str,
}

/// This service's liveness. It needs nothing, so it cannot fail.
pub fn check() -> Status {
    Status { status: "ok" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_ready() {
        assert_eq!(check().status, "ok");
    }
}
