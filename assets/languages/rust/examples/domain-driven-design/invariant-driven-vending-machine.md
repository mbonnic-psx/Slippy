```rust
// ❌ WRONG: a relationship-driven aggregate mirroring the entity hierarchy. No invariant is enforced; its
// methods only manage associations.
pub struct Route {
    pub id: RouteId,
    pub locations: Vec<Location>, // why is this here?
    // add_location(...)          just manages a collection
    // attach_vending_machine(...) just manages a relationship
    // alarm_count()              a read concern leaking in
}

// ✅ RIGHT: its own aggregate, because alarms are the behavioural responsibility.
pub struct VendingMachine {
    pub id: VendingMachineId,
    pub location_id: LocationId, // reference by id
    pub alarms: Vec<Alarm>,
    pub max_concurrent_alarms: usize, // invariant: cannot exceed this
}

#[derive(Debug, PartialEq)]
pub enum TriggerAlarmRejection {
    MaxAlarmsReached,
}

/// The invariant that justifies this aggregate.
pub fn trigger_alarm(machine: VendingMachine, alarm: NewAlarm) -> Result<VendingMachine, TriggerAlarmRejection> {
    let active = machine.alarms.iter().filter(|a| a.status == AlarmStatus::Active).count();
    if active >= machine.max_concurrent_alarms {
        return Err(TriggerAlarmRejection::MaxAlarmsReached);
    }
    let mut alarms = machine.alarms;
    alarms.push(Alarm { id: alarm.id, status: alarm.status });
    Ok(VendingMachine { alarms, ..machine })
}
```
