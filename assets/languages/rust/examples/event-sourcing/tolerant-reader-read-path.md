```rust
/// The read-time pipeline: validate the raw stored JSON against the shape that was actually persisted
/// (possibly an old version), then upcast it to the current shape. Validate first, then upcast — never let
/// unvalidated data reach the upcaster.
///
/// `StoredAccountEvent` is a tolerant reader over the union of every explicitly persisted version: unknown
/// fields are ignored, and only fields that were always optional (or have a proven, context-invariant
/// default) may be absent. It fails on genuinely corrupt data — a bug, not a business case — which the caller
/// handles.
///
/// `upcast_account_event` then maps whatever validated, possibly old shape came back onto the current
/// `AccountEvent` (the OrderPlaced upcaster shows one such step in full).
pub fn to_domain_event(raw: &[u8]) -> Result<AccountEvent, ReadError> {
    let stored: StoredAccountEvent = serde_json::from_slice(raw)?;
    upcast_account_event(stored)
}
```
