```rust
// ❌ WRONG: a bare string error tells you nothing. Nothing stops two call sites spelling the same failure two
// ways, and the compiler cannot check that every case is handled. Use a specific reason type instead — see
// `PledgeDecision`.
pub struct OperationResult {
    pub success: bool,
    pub data: Option<Box<dyn std::any::Any>>,
    pub error: String,
}
```
