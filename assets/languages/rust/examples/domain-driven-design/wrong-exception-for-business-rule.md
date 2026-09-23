```rust
// ❌ WRONG: panics for what should be an expected business outcome. The failure is invisible in the
// signature, and nothing forces a caller to handle it — the service simply falls over.
pub fn pledge_contribution(occasion: Occasion, eligibility: &Eligibility, pledge: NewPledge) -> Occasion {
    if occasion.is_funding_closed {
        panic!("funding is closed");
    }
    // ...
}
```
