use chrono::Days;
use chrono::NaiveDate;

use super::super::model::option::*;
use super::super::model::psp::*;
use super::super::model::rsu::*;
use super::*;

pub struct ValuationItem {
    date: NaiveDate,
    psp: i32,
    options_vested_total: i32,
    options_unvested_total: i32,
    rsu_vested_units: i32,
    rsu_vested_total: i32,
    rsu_unvested_units: i32,
    rsu_unvested_total: i32,
    vested_total: i32,
    unvested_total: i32,
    grant_total: i32,
}

pub struct Valuation {
    items: Vec<ValuationItem>,
}

pub struct VestedOptionBundle {
    number: i32,
    exercise_price: i32,
}

impl Valuation {
    pub fn new(
        psp: &PreferredStockPrice,
        option_grants: &Vec<OptionGrant>,
        rsu_grants: &Vec<RestrictedStockUnitGrant>,
    ) -> Valuation {
        // When vesting starts
        let start_date = rsu_grants
            .iter()
            .map(|rsu_grant| rsu_grant.vesting_schedule.commences_on)
            .chain(
                option_grants
                    .iter()
                    .map(|option_grant| option_grant.vesting_schedule.commences_on),
            )
            .min()
            .unwrap();

        // When vested
        let end_date = rsu_grants
            .iter()
            .map(|rsu_grant| rsu_grant.vesting_schedule.events.last().unwrap().date)
            .chain(
                option_grants
                    .iter()
                    .map(|option_grant| option_grant.vesting_schedule.events.last().unwrap().date),
            )
            .max()
            .unwrap();

        let mut cursor = start_date;
        let mut days: Vec<ValuationItem> = Vec::new();
        let mut rsu_vested_units = 0;
        let mut rsu_unvested_units = 0;
        let mut options_vested_units: Vec<VestedOptionBundle> = vec![];
        let mut options_unvested_units: Vec<VestedOptionBundle> = vec![];

        while cursor <= end_date {
            for rsu_grant in rsu_grants {
                if rsu_grant.granted_on == cursor {
                    rsu_unvested_units += rsu_grant.actual_total_units();
                }

                for event in &rsu_grant.vesting_schedule.events {
                    if event.date == cursor {
                        rsu_vested_units += event.number;
                        rsu_unvested_units -= event.number;
                    }
                }
            }

            for option_grant in option_grants {
                if option_grant.granted_on == cursor {
                    options_unvested_units.push(VestedOptionBundle {
                        number: option_grant.value.number,
                        exercise_price: option_grant.value.exercise_price,
                    });
                }

                for event in &option_grant.vesting_schedule.events {
                    if event.date == cursor {
                        options_vested_units.push(VestedOptionBundle {
                            number: event.number,
                            exercise_price: option_grant.value.exercise_price,
                        });

                        // Remove unvested options by adding a negative amount
                        options_unvested_units.push(VestedOptionBundle {
                            number: -1 * event.number,
                            exercise_price: option_grant.value.exercise_price,
                        });
                    }
                }
            }

            let psp_on = psp.value_on(&cursor);
            let rsu_vested_total = rsu_vested_units * psp_on;
            let rsu_unvested_total = rsu_unvested_units * psp_on;

            let options_vested_total: i32 = options_vested_units
                .iter()
                .map(|bundle| bundle.number * (psp_on - bundle.exercise_price))
                .sum();

            let options_unvested_total: i32 = options_unvested_units
                .iter()
                .map(|bundle| bundle.number * (psp_on - bundle.exercise_price))
                .sum();

            let vested_total = rsu_vested_total + options_vested_total;
            let unvested_total = rsu_unvested_total + options_unvested_total;

            let grant_total = unvested_total + vested_total;

            days.push(ValuationItem {
                date: cursor.clone(),
                psp: psp_on,
                options_unvested_total,
                options_vested_total,
                rsu_vested_total,
                rsu_vested_units,
                rsu_unvested_total,
                rsu_unvested_units,
                unvested_total,
                vested_total,
                grant_total,
            });

            cursor = cursor.checked_add_days(Days::new(1)).unwrap();
        }

        Valuation { items: days }
    }

    pub fn print_to_csv(&self) {
        println!("Date,Preferred Stock Price,Options Vested Total, Options Unvested Total, RSUs Vested Total,RSUs Unvested Total,Vested Total,Unvested Total,Grand Total");

        for item in &self.items {
            println!(
                "{},{},{},{},{},{},{},{},{}",
                item.date,
                format_currency(item.psp),
                format_currency(item.options_vested_total),
                format_currency(item.options_unvested_total),
                format_currency(item.rsu_vested_total),
                format_currency(item.rsu_unvested_total),
                format_currency(item.vested_total),
                format_currency(item.unvested_total),
                format_currency(item.grant_total)
            );
        }
    }
}
