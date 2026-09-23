```rust
// ❌ WRONG: wraps an error only to prepend a message — noise, not clarity, when the error underneath already
// carries enough context. `?` alone would say the same thing.
pub fn handle_pledge(occasion: Occasion, eligibility: &Eligibility, pledge: NewPledge) -> Result<(), anyhow::Error> {
    pledge_contribution(occasion, eligibility, pledge).map_err(|error| anyhow::anyhow!("failed to pledge: {error}"))?;
    Ok(())
}
```
