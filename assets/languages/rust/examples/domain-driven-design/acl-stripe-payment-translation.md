```rust
/// Our domain's identity for a completed charge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChargeId(String);

impl ChargeId {
    pub fn new(raw: &str) -> Result<Self, AclError> {
        if raw.is_empty() {
            return Err(AclError::EmptyChargeId);
        }
        Ok(Self(raw.to_owned()))
    }
}

/// An amount in integer minor units — never floating point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Money {
    pub minor_units: i64,
    pub currency: String,
}

/// The external system's model, exactly as Stripe returns it.
#[derive(Debug, serde::Deserialize)]
pub struct StripeCharge {
    pub id: String,
    pub amount: i64,      // integer minor units, per Stripe's contract
    pub currency: String, // lower-case ISO code
    pub status: String,
}

/// Our domain's outcome, decoupled from Stripe's vocabulary.
#[derive(Debug, PartialEq)]
pub enum PaymentResult {
    Succeeded { charge_id: ChargeId, amount: Money },
    Failed { reason: String },
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum AclError {
    #[error("translating stripe charge: charge id must not be empty")]
    EmptyChargeId,
}

/// The anti-corruption layer: it translates Stripe's model into ours at the boundary, so no Stripe vocabulary
/// leaks in.
impl TryFrom<StripeCharge> for PaymentResult {
    type Error = AclError;

    fn try_from(charge: StripeCharge) -> Result<Self, AclError> {
        if charge.status == "succeeded" {
            return Ok(PaymentResult::Succeeded {
                charge_id: ChargeId::new(&charge.id)?,
                amount: Money { minor_units: charge.amount, currency: charge.currency.to_uppercase() },
            });
        }
        Ok(PaymentResult::Failed { reason: format!("payment failed: {}", charge.status) })
    }
}
```
