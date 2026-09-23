```rust
/// The child collection is private, so the aggregate root controls every change. Callers get a slice — a
/// read-only borrow they can inspect but, by the type system's say-so, never mutate.
pub struct Workout {
    pub id: WorkoutId,
    exercises: Vec<Exercise>,
    pub max_exercises: usize,
}

impl Workout {
    pub fn exercises(&self) -> &[Exercise] {
        &self.exercises
    }
}
```
