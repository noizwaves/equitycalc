use chrono::{Datelike, Days, Months, NaiveDate};

use crate::{
    model::psp::PreferredStockPrice,
    model::{option::OptionGrant, psp::*, rsu::RestrictedStockUnitGrant},
    report::format_currency,
};

#[derive(Debug)]
pub struct ReportLine {
    from: NaiveDate,
    to: NaiveDate,
    total: i32,
    by_grant: Vec<i32>,
}

pub struct Report {
    grant_names: Vec<String>,
    lines: Vec<ReportLine>,
}

fn start_of_quarter(date: &NaiveDate) -> NaiveDate {
    // map month to quarter
    // 1 -> 1
    // 3 -> 1
    // 4 -> 2
    // etc
    let quarter = (date.month() - 1) / 3 + 1;

    // map quarter to first month
    // 1 -> 1
    // 2 -> 4
    // 3 -> 7
    let first_month = (quarter * 3) - 2;

    NaiveDate::from_ymd_opt(date.year(), first_month, 1).unwrap()
}

impl Report {
    pub fn new(
        psp: &PreferredStockPrice,
        option_grants: &Vec<OptionGrant>,
        rsu_grants: &Vec<RestrictedStockUnitGrant>,
    ) -> Report {
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

        let start_quarter_date = start_of_quarter(&start_date);

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

        // Vesting starts quarter start date
        let end_quarter_date = start_of_quarter(&end_date);

        let mut cursor = start_quarter_date;
        let mut lines: Vec<ReportLine> = Vec::new();

        let grant_names = option_grants
            .iter()
            .map(|grant| grant.name.clone())
            .chain(rsu_grants.iter().map(|grant| grant.name.clone()))
            .collect();

        while cursor <= end_quarter_date {
            let from = cursor;
            let to = from
                .checked_add_months(Months::new(3))
                .unwrap()
                .checked_sub_days(Days::new(1))
                .unwrap();

            let mut total = 0;

            let mut by_grant: Vec<i32> = Vec::new();
            for grant in option_grants {
                let grant_total = &grant
                    .vesting_schedule
                    .events
                    .iter()
                    .filter(|event| event.date >= from && event.date <= to)
                    .map(|event| {
                        let unit_value = psp.value_on(&event.date) - grant.value.exercise_price;
                        event.number * unit_value
                    })
                    .sum();
                total += grant_total;
                by_grant.push(*grant_total);
            }

            for grant in rsu_grants {
                let grant_total = &grant
                    .vesting_schedule
                    .events
                    .iter()
                    .filter(|event| event.date >= from && event.date <= to)
                    .map(|event| {
                        let unit_value = psp.value_on(&event.date);
                        event.number * unit_value
                    })
                    .sum();
                total += grant_total;
                by_grant.push(*grant_total);
            }

            lines.push(ReportLine {
                from,
                to,
                total,
                by_grant,
            });

            cursor = cursor.checked_add_months(Months::new(3)).unwrap();
        }

        Report { grant_names, lines }
    }

    pub fn print_to_csv(&self) {
        println!(
            "Quarter Start,Quarter End,{},Total",
            &self.grant_names.join(",")
        );
        for line in &self.lines {
            println!(
                "{},{},{},{}",
                line.from,
                line.to,
                line.by_grant
                    .iter()
                    .map(|v| format_currency(*v))
                    .collect::<Vec<String>>()
                    .join(","),
                format_currency(line.total)
            );
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_start_of_quarter() {
        assert_eq!(
            NaiveDate::from_ymd_opt(2023, 10, 1).unwrap(),
            start_of_quarter(&NaiveDate::from_ymd_opt(2023, 12, 31).unwrap())
        );

        assert_eq!(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            start_of_quarter(&NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())
        );

        assert_eq!(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            start_of_quarter(&NaiveDate::from_ymd_opt(2024, 3, 31).unwrap())
        );

        assert_eq!(
            NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
            start_of_quarter(&NaiveDate::from_ymd_opt(2024, 4, 1).unwrap())
        );
    }
}
