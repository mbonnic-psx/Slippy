```rust
// ❌ WRONG: query concerns leak into the aggregate.
pub struct Route {
    pub id: RouteId,
    pub locations: Vec<Location>,
    pub alarm_count: u32,                        // read concern — supports no invariant
    pub last_alarm_date: Option<OffsetDateTime>, // read concern — no command needs this
}

// ✅ RIGHT: only what commands need to enforce invariants.
pub struct VendingMachine {
    pub id: VendingMachineId,
    pub location_id: LocationId,
    pub alarms: Vec<Alarm>,           // needed for the max-alarms invariant
    pub max_concurrent_alarms: usize, // the invariant itself
}

// ✅ RIGHT: a read model that answers query-side questions independently of the aggregate.
pub struct AlarmSummaryView {
    pub route_id: RouteId,
    pub total_alarms: u32,
    pub last_alarm_at: Option<OffsetDateTime>,
    pub active_alarm_count: u32,
}
```
