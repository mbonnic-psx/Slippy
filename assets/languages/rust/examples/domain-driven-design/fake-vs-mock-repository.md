```rust
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: UserId,
    pub email: String,
}

/// The port both examples below implement.
pub trait UserRepository {
    fn find_by_id(&self, id: &UserId) -> Result<Option<User>, RepositoryError>;
    fn save(&self, user: &User) -> Result<(), RepositoryError>;
}

/// Good: a fake — real state behind the real trait. A test built on it exercises the contract production
/// code uses, so it proves the use case works, not merely that a call happened.
#[derive(Default)]
pub struct FakeUserRepository {
    users: RefCell<HashMap<UserId, User>>,
}

impl UserRepository for FakeUserRepository {
    fn find_by_id(&self, id: &UserId) -> Result<Option<User>, RepositoryError> {
        Ok(self.users.borrow().get(id).cloned())
    }

    fn save(&self, user: &User) -> Result<(), RepositoryError> {
        self.users.borrow_mut().insert(user.id.clone(), user.clone());
        Ok(())
    }
}

/// Bad: a "mock" that only records whether a method was called. It knows nothing about state, so it cannot
/// catch "saved the wrong user" or "saved twice" — it can only prove a call happened, which is rarely what
/// the business rule requires.
#[derive(Default)]
struct MockUserRepository {
    save_called: Cell<bool>,
}

impl UserRepository for MockUserRepository {
    fn find_by_id(&self, _id: &UserId) -> Result<Option<User>, RepositoryError> {
        Ok(None)
    }

    fn save(&self, _user: &User) -> Result<(), RepositoryError> {
        self.save_called.set(true);
        Ok(())
    }
}
```
