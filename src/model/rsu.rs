use chrono::NaiveDate;

#[derive(PartialEq, Debug)]
pub struct RestrictedStockUnitValue {
    pub grant_price_cents: i32,
    pub total_value_cents: i32,
}

impl RestrictedStockUnitValue {
    pub fn new(grant_price_cents: i32, total_value_cents: i32) -> RestrictedStockUnitValue {
        RestrictedStockUnitValue {
            grant_price_cents,
            total_value_cents,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct RestrictedStockUnitVestingEvent {
    pub date: NaiveDate,
    pub number: i32,
}

impl RestrictedStockUnitVestingEvent {
    pub fn new(date: NaiveDate, number: i32) -> RestrictedStockUnitVestingEvent {
        RestrictedStockUnitVestingEvent { date, number }
    }
}

#[derive(PartialEq, Debug)]
pub struct RestrictedStockUnitVestingSchedule {
    pub commences_on: NaiveDate,
    pub events: Vec<RestrictedStockUnitVestingEvent>,
}

impl RestrictedStockUnitVestingSchedule {
    pub fn new(
        commences_on: NaiveDate,
        events: Vec<RestrictedStockUnitVestingEvent>,
    ) -> RestrictedStockUnitVestingSchedule {
        let mut events = events.clone();
        events.sort_by(|a, b| a.date.cmp(&b.date));
        RestrictedStockUnitVestingSchedule {
            commences_on,
            events,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct RestrictedStockUnitGrant {
    pub name: String,
    pub granted_on: NaiveDate,
    pub value: RestrictedStockUnitValue,
    pub vesting_schedule: RestrictedStockUnitVestingSchedule,
}

impl RestrictedStockUnitGrant {
    pub fn new(
        name: String,
        granted_on: NaiveDate,
        value: RestrictedStockUnitValue,
        vesting_schedule: RestrictedStockUnitVestingSchedule,
    ) -> RestrictedStockUnitGrant {
        RestrictedStockUnitGrant {
            name,
            granted_on,
            value,
            vesting_schedule,
        }
    }

    pub fn actual_total_units(&self) -> i32 {
        self.vesting_schedule
            .events
            .iter()
            .map(|event| event.number)
            .sum()
    }
}
