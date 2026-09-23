```rust
use std::collections::HashMap;
use std::sync::Mutex;

/// The driven port both adapters below implement.
pub trait UserRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, RepositoryError>;
    async fn save(&self, user: &User) -> Result<(), RepositoryError>;
}

/// The driven adapter, backed by sqlx.
pub struct PostgresUserRepository {
    pool: sqlx::PgPool,
}

impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, RepositoryError> {
        let row = sqlx::query_as::<_, UserRow>(FIND_USER_BY_ID).bind(id).fetch_optional(&self.pool).await?;
        Ok(row.map(User::from))
    }

    async fn save(&self, user: &User) -> Result<(), RepositoryError> {
        sqlx::query(UPSERT_USER).bind(&user.id).bind(&user.email).execute(&self.pool).await?;
        Ok(())
    }
}

/// The same port in memory, for tests.
#[derive(Default)]
pub struct FakeUserRepository {
    users: Mutex<HashMap<String, User>>,
}

impl FakeUserRepository {
    pub fn with(users: impl IntoIterator<Item = User>) -> Self {
        Self { users: Mutex::new(users.into_iter().map(|user| (user.id.clone(), user)).collect()) }
    }
}

impl UserRepository for FakeUserRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, RepositoryError> {
        Ok(self.users.lock().expect("fake lock").get(id).cloned())
    }

    async fn save(&self, user: &User) -> Result<(), RepositoryError> {
        self.users.lock().expect("fake lock").insert(user.id.clone(), user.clone());
        Ok(())
    }
}
```
