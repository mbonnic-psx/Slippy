```rust
/// A queue deployment entry point: inline composition plus the driving adapter.
pub async fn handle_pledge_message(message: SqsMessage, env: &Env) -> Result<(), HandleError> {
    let pool = open_pool(&env.database_url).await?;
    let occasions = PostgresOccasionRepository::new(pool.clone());
    let contributors = PostgresContributorRepository::new(pool);
    let pledging = PledgingToOccasions::new(occasions, contributors);

    let dto: PledgeDto = serde_json::from_str(&message.body).map_err(HandleError::Parse)?;
    let mut command = dto.into_command();
    command.pledge_id = PledgeId::from_message(&message.message_id);

    pledging.pledge_to_occasion(command).await?;
    Ok(())
}
```
