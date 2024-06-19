use chrono::{format, Days, NaiveDate};

mod portfolio;

struct RestrictedStockUnitValue {
    grant_price_cents: i32,
    total_value_cents: i32,
}

struct RestrictedStockUnitVestingEvent {
    date: NaiveDate,
    number: i32,
}

struct RestrictedStockUnitVestingSchedule {
    events: Vec<RestrictedStockUnitVestingEvent>,
}

struct RestrictedStockUnitGrant {
    name: String,
    granted_on: NaiveDate,
    value: RestrictedStockUnitValue,
    vesting_schedule: RestrictedStockUnitVestingSchedule,
}

impl RestrictedStockUnitGrant {
    fn actual_total_value(&self) -> i32 {
        self.vesting_schedule
            .events
            .iter()
            .map(|event| event.number * self.value.grant_price_cents)
            .sum()
    }

    fn actual_total_units(&self) -> i32 {
        self.vesting_schedule
            .events
            .iter()
            .map(|event| event.number)
            .sum()
    }
}

#[derive(Debug, Clone)]
struct PreferredStockPriceValuation {
    date: NaiveDate,
    value_cents: i32,
}

#[derive(Debug)]
struct PreferredStockPrice {
    values: Vec<PreferredStockPriceValuation>,
}

impl PreferredStockPrice {
    fn new(values: Vec<PreferredStockPriceValuation>) -> PreferredStockPrice {
        let mut sorted = values.clone();
        sorted.sort_by(|a, b| a.date.cmp(&b.date));

        PreferredStockPrice { values: sorted }
    }

    fn value_on(&self, date: &NaiveDate) -> i32 {
        // The first valuation after `date`
        self.values
            .iter()
            .take_while(|valuation| &valuation.date <= date)
            .last()
            .expect(&format!("No valuation found for {date}"))
            .value_cents
    }
}

struct ValuationItem {
    date: NaiveDate,
    psp: i32,
    vested_units: i32,
    vested_total: i32,
    unvested_units: i32,
    unvested_total: i32,
    total_value: i32,
}

struct Valuation {
    items: Vec<ValuationItem>,
}

fn format_currency(cents: i32) -> String {
    format!("{0}.{1:2>0}", cents / 100, cents % 100)
}

impl Valuation {
    fn new(psp: &PreferredStockPrice, rsu_grants: &Vec<RestrictedStockUnitGrant>) -> Valuation {
        // Find the starting date (the earliest grant date)
        let start_date = rsu_grants
            .iter()
            .map(|rsu_grant| rsu_grant.granted_on)
            .min()
            .unwrap();

        // Find the last vesting date
        let end_date = rsu_grants
            .iter()
            .map(|rsu_grant| rsu_grant.vesting_schedule.events.last().unwrap().date)
            .max()
            .unwrap();

        let mut cursor = start_date.clone();
        let mut days: Vec<ValuationItem> = Vec::new();
        let mut vested_units = 0;
        let mut unvested_units = 0;

        while cursor <= end_date {
            for rsu_grant in rsu_grants {
                if rsu_grant.granted_on == cursor {
                    unvested_units += rsu_grant.actual_total_units();
                }

                for event in &rsu_grant.vesting_schedule.events {
                    if event.date == cursor {
                        vested_units += event.number;
                        unvested_units -= event.number;
                    }
                }
            }

            let psp_on = psp.value_on(&cursor);
            let vested_value = vested_units * psp_on;
            let unvested_value = unvested_units * psp_on;
            let total_value = vested_value + unvested_value;

            days.push(ValuationItem {
                date: cursor.clone(),
                psp: psp_on,
                vested_total: vested_value,
                vested_units,
                unvested_total: unvested_value,
                unvested_units,
                total_value,
            });

            cursor = cursor.checked_add_days(Days::new(1)).unwrap();
        }

        Valuation { items: days }
    }

    fn print_to_csv(&self) {
        println!(
            "Date,Preferred Stock Price,Vested Units,Vested Total,Unvested Units,Unvested Total,Grant Total"
        );

        for item in &self.items {
            println!(
                "{},{},{},{},{},{},{}",
                item.date,
                format_currency(item.psp),
                item.vested_units,
                format_currency(item.vested_total),
                item.unvested_units,
                format_currency(item.unvested_total),
                format_currency(item.total_value)
            );
        }
    }
}

fn main() {
    let valuation = Valuation::new(
        &portfolio::preferred_stock_price(),
        &portfolio::restricted_stock_grants(),
    );

    valuation.print_to_csv();
}
