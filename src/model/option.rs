use chrono::NaiveDate;

pub struct OptionGrantValue {
    pub exercise_price: i32,
    pub number: i32,
}

impl OptionGrantValue {
    pub fn new(exercise_price: i32, number: i32) -> OptionGrantValue {
        OptionGrantValue {
            exercise_price,
            number,
        }
    }
}

#[derive(Clone)]
pub struct OptionGrantVestingEvent {
    pub date: NaiveDate,
    pub number: i32,
}

impl OptionGrantVestingEvent {
    pub fn new(date: NaiveDate, number: i32) -> OptionGrantVestingEvent {
        OptionGrantVestingEvent { date, number }
    }
}

pub struct OptionGrantVestingSchedule {
    pub events: Vec<OptionGrantVestingEvent>,
}

impl OptionGrantVestingSchedule {
    pub fn new(events: Vec<OptionGrantVestingEvent>) -> OptionGrantVestingSchedule {
        let mut events = events.clone();
        events.sort_by(|a, b| a.date.cmp(&b.date));
        OptionGrantVestingSchedule { events }
    }
}

pub struct OptionGrant {
    pub name: String,
    pub granted_on: NaiveDate,
    pub value: OptionGrantValue,
    pub vesting_schedule: OptionGrantVestingSchedule,
}

impl OptionGrant {
    pub fn new(
        name: String,
        granted_on: NaiveDate,
        value: OptionGrantValue,
        vesting_schedule: OptionGrantVestingSchedule,
    ) -> OptionGrant {
        OptionGrant {
            name,
            granted_on,
            value,
            vesting_schedule,
        }
    }
}
