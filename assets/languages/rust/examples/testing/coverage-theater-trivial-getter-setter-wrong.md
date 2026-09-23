```rust
#[test]
fn worker_set_retry_limit() {
    let mut worker = Worker::default();

    worker.set_retry_limit(3);

    assert_eq!(worker.retry_limit(), 3); // trivial
}
```
