# The application layer

What this service *does*, said once: the use cases — command handlers, query handlers, projection
definitions — and the ports they are driven through.

Ports go under `ports/`, one module each. Where this project was given an event store, the port every write
goes through is already there. Modules under `crate::adapters::driven` implement a port; modules under
`crate::adapters::driving` call a use case. Neither is used from here.

**What belongs here.** A use case that orchestrates: load what the decision needs through a port, call the
domain, append what it returned. A port is a trait named for what the application needs, never for what
implements it — declared here, beside its consumer, not beside its implementer.

**What it may not use**, enforced by `make check-imports`:

- anything under `crate::adapters`, `crate::composition`, `crate::infrastructure` or `crate::delivery`. An
  adapter is injected, never reached for, and the composition root is the only place a port and its
  implementation meet.

It may use the domain freely: that is the direction the whole rule exists to keep.
