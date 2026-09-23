```rust
//! The driving adapter at the HTTP entry point: gifting/src/adapters/driving/http/occasions_pledge.rs

/// A strict schema: only `amount` is accepted. Contributor or tenant fields in the body are refused, never
/// trusted from the request.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PledgeBody {
    amount: AmountBody,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AmountBody {
    minor_units: u32,
    currency: Currency, // deserialises only GBP, USD or EUR
}

/// The executable entry point for `POST /occasions/{id}/pledge`. It combines a driving adapter with inline
/// composition because this is the deployment entry point and its graph is trivial; a larger graph belongs in
/// an explicit composition root.
pub async fn post_pledge(
    State(pool): State<sqlx::PgPool>,
    // The authentication extractor is the only production constructor of this provider-free principal; the
    // request body cannot choose the actor, and a request without one is a 401 before this body runs.
    principal: AuthenticatedPledger,
    Path(raw_id): Path<String>,
    body: Bytes,
) -> Response {
    let Some(occasion_id) = parse_occasion_id(&raw_id) else {
        return json(StatusCode::UNPROCESSABLE_ENTITY, json!({ "error": "invalid-path" }));
    };
    // Malformed transport syntax is a 400; syntactically valid JSON that breaks the schema is a 422.
    let Ok(parsed) = serde_json::from_slice::<Value>(&body) else {
        return json(StatusCode::BAD_REQUEST, json!({ "error": "malformed-json" }));
    };
    let Ok(PledgeBody { amount }) = serde_json::from_value::<PledgeBody>(parsed) else {
        return json(StatusCode::UNPROCESSABLE_ENTITY, json!({ "error": "invalid-body" }));
    };

    // Inline composition: this handler is the executable entry point and the graph is trivial.
    let pledging = PledgingToOccasions { persistence: PostgresPledgePersistence::new(pool) };

    let command = PledgeToOccasionCommand {
        pledge_id: PledgeId(Uuid::new_v4().to_string()),
        occasion_id,
        principal,
        amount: Money { minor_units: i64::from(amount.minor_units), currency: amount.currency },
    };
    let result = match pledging.pledge_to_occasion(command).await {
        Ok(result) => result,
        Err(_) => return json(StatusCode::INTERNAL_SERVER_ERROR, json!({ "error": "internal-error" })),
    };

    // Status selection is protocol translation, not business policy.
    match result {
        Ok(pledged) => json(StatusCode::OK, json!({ "pledged": pledged.occasion.total_pledged })),
        Err(reason) => {
            let status = match reason {
                PledgeRejection::NotFound => StatusCode::NOT_FOUND,
                PledgeRejection::ConcurrentChange => StatusCode::CONFLICT,
                _ => StatusCode::UNPROCESSABLE_ENTITY,
            };
            json(status, json!({ "error": reason }))
        }
    }
}

fn parse_occasion_id(raw: &str) -> Option<OccasionId> {
    let trimmed = raw.trim();
    (!trimmed.is_empty()).then(|| OccasionId(trimmed.to_owned()))
}

fn json(status: StatusCode, body: Value) -> Response {
    (status, Json(body)).into_response()
}
```
