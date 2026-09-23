```rust
// ❌ WRONG: leaks internals — the caller must obtain the Exercise somehow.
pub fn remove_exercise(workout: Workout, exercise: Exercise) -> Workout {
    unimplemented!("wrong shape — see the version below")
}

#[derive(Debug, PartialEq)]
pub enum RemoveExerciseRejection {
    ExerciseNotFound,
}

// ✅ RIGHT: preserves the boundary — the caller knows only the id, and the root looks the child up itself.
pub fn remove_exercise(workout: Workout, exercise_id: &ExerciseId) -> Result<Workout, RemoveExerciseRejection> {
    let before = workout.exercises.len();
    let exercises: Vec<Exercise> = workout.exercises.into_iter().filter(|e| e.id != *exercise_id).collect();
    if exercises.len() == before {
        return Err(RemoveExerciseRejection::ExerciseNotFound);
    }
    Ok(Workout { exercises, ..workout })
}
```
