```rust
// ❌ WRONG: accepts an externally built child — the root cannot enforce its own creation rules.
pub fn add_exercise(workout: Workout, exercise: Exercise) -> Workout {
    unimplemented!("wrong shape — see the version below")
}

#[derive(Debug, PartialEq)]
pub enum AddExerciseRejection {
    MaxExercisesReached,
}

// ✅ RIGHT: the root creates the child itself, enforcing the max-exercises invariant.
pub fn add_exercise(workout: Workout, exercise_id: ExerciseId, params: NewExerciseParams) -> Result<Workout, AddExerciseRejection> {
    if workout.exercises.len() >= workout.max_exercises {
        return Err(AddExerciseRejection::MaxExercisesReached);
    }
    let exercise = Exercise {
        id: exercise_id,
        workout_id: workout.id.clone(),
        name: params.name,
        target_sets: params.target_sets,
        target_reps: params.target_reps,
    };
    let mut exercises = workout.exercises;
    exercises.push(exercise);
    Ok(Workout { exercises, ..workout })
}
```
