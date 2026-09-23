```rust
// deduct_user_balance.rs — driving port + use case

/// Opaque proof of authentication; the request body cannot select a user.
#[derive(Debug, Clone)]
pub struct AuthenticatedPrincipal {
    pub user_id: String,
}

pub struct DeductUserBalanceCommand {
    pub principal: AuthenticatedPrincipal,
    pub amount: Money,
}

#[derive(Debug, PartialEq)]
pub enum DeductUserBalanceOutcome {
    Succeeded(User),
    NotFound,
    ConcurrentChange,
    NonPositiveAmount,
    CurrencyMismatch,
    InsufficientBalance,
}

/// The driving port this use case satisfies.
pub trait ForDeductingUserBalances {
    async fn deduct_user_balance(&self, command: DeductUserBalanceCommand)
        -> Result<DeductUserBalanceOutcome, RepositoryError>;
}

/// Wires the pure domain rule to the repository port.
pub struct UserBalanceDeduction<R> {
    pub users: R,
}

impl<R: UserRepository> ForDeductingUserBalances for UserBalanceDeduction<R> {
    async fn deduct_user_balance(
        &self,
        command: DeductUserBalanceCommand,
    ) -> Result<DeductUserBalanceOutcome, RepositoryError> {
        let Some(stored) = self.users.find_by_id(&command.principal.user_id).await? else {
            return Ok(DeductUserBalanceOutcome::NotFound);
        };

        let user = match deduct_balance(stored.value, command.amount) {
            Ok(user) => user,
            Err(DeductRejection::NonPositiveAmount) => return Ok(DeductUserBalanceOutcome::NonPositiveAmount),
            Err(DeductRejection::CurrencyMismatch) => return Ok(DeductUserBalanceOutcome::CurrencyMismatch),
            Err(DeductRejection::InsufficientBalance) => return Ok(DeductUserBalanceOutcome::InsufficientBalance),
        };

        match self.users.save(&user, stored.version).await? {
            SaveOutcome::Saved => Ok(DeductUserBalanceOutcome::Succeeded(user)),
            SaveOutcome::Conflict => Ok(DeductUserBalanceOutcome::ConcurrentChange),
        }
    }
}
```
