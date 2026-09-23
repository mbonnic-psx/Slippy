// The migrations are compiled into `bin/migrate` by `sqlx::migrate!`, and Cargo does not know that: without
// this, a migration added to the directory would not reach the binary until something else changed.
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
