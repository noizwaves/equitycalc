use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use chrono::{Months, NaiveDate};
use serde::Deserialize;
use serde_yaml::Deserializer;

use crate::model;

mod naive_date_format {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer};

    const FORMAT: &str = "%Y-%m-%d";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

pub type Result<T> = anyhow::Result<T>;

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("unable to load file at {0}")]
    FileLoadFailed(PathBuf),

    #[error("deserialization from expected YAML structure failed")]
    DeserializationFailed(#[from] serde_yaml::Error),
}

#[derive(Debug, Deserialize)]
struct PreferredStockPrice {
    #[serde(with = "naive_date_format")]
    date: NaiveDate,
    price: f64,
}

impl PreferredStockPrice {
    pub fn to_model(&self) -> model::psp::PreferredStockPriceValuation {
        model::psp::PreferredStockPriceValuation::new(self.date, (self.price * 100.0) as i32)
    }
}

pub fn load_psp(portfolio_path: &Path) -> Result<model::psp::PreferredStockPrice> {
    let psp_path = portfolio_path.join("psp.yaml");

    let contents =
        fs::read_to_string(&psp_path).map_err(|_| LoadError::FileLoadFailed(psp_path.clone()))?;

    let mut result: Vec<PreferredStockPrice> = Vec::new();
    for doc in Deserializer::from_str(&contents) {
        let psp = PreferredStockPrice::deserialize(doc)
            .map_err(LoadError::DeserializationFailed)
            .with_context(|| {
                format!(
                    "Preferred Stock Price deserialize failed from {:?}",
                    &psp_path
                )
            })?;

        result.push(psp);
    }

    let valuations = result.into_iter().map(|p| p.to_model()).collect();

    Ok(model::psp::PreferredStockPrice::new(valuations))
}

impl OptionGrant {
    pub fn to_model(&self) -> model::option::OptionGrant {
        let vesting_events = self
            .vesting_schedule
            .events
            .iter()
            .map(|e| model::option::OptionGrantVestingEvent::new(e.date, e.number_of_shares))
            .collect();

        model::option::OptionGrant::new(
            self.name.clone(),
            self.date,
            model::option::OptionGrantValue::new(
                (self.grant_value.exercise_price * 100.0) as i32,
                self.grant_value.shares,
            ),
            model::option::OptionGrantVestingSchedule::new(
                self.vesting_schedule.commences_on,
                vesting_events,
            ),
        )
    }
}

#[derive(Debug, Deserialize)]
struct OptionGrant {
    name: String,

    #[serde(with = "naive_date_format")]
    date: NaiveDate,
    grant_value: OptionGrantValue,
    vesting_schedule: OptionGrantVestingSchedule,
}

#[derive(Debug, Deserialize)]
struct OptionGrantValue {
    exercise_price: f32,
    shares: i32,
}

#[derive(Debug, Deserialize)]
struct OptionGrantVestingSchedule {
    #[serde(with = "naive_date_format")]
    commences_on: NaiveDate,
    events: Vec<OptionGrantVestingEvent>,
}

#[derive(Debug, Deserialize)]
struct OptionGrantVestingEvent {
    #[serde(with = "naive_date_format")]
    date: NaiveDate,
    number_of_shares: i32,
}

pub fn load_option_grants(portfolio_path: &Path) -> Result<Vec<model::option::OptionGrant>> {
    let grants_path = portfolio_path.join("option_grants.yaml");
    let contents = fs::read_to_string(&grants_path)
        .map_err(|_| LoadError::FileLoadFailed(grants_path.clone()))?;

    let mut result: Vec<OptionGrant> = Vec::new();
    for doc in Deserializer::from_str(&contents) {
        let grant = OptionGrant::deserialize(doc)
            .map_err(LoadError::DeserializationFailed)
            .with_context(|| format!("Options Grant deserialize failed from {:?}", &grants_path))?;
        result.push(grant);
    }

    Ok(result.into_iter().map(|g| g.to_model()).collect())
}

#[derive(Debug, Deserialize)]
struct RestrictedStockUnitGrant {
    name: String,

    #[serde(with = "naive_date_format")]
    date: NaiveDate,
    grant_value: RestrictedStockUnitGrantValue,
    vesting: RestrictedStockUnitVesting,
}

impl RestrictedStockUnitGrant {
    pub fn to_model(&self) -> model::rsu::RestrictedStockUnitGrant {
        let events = match &self.vesting.implementation {
            RestrictedStockUnitVestingDefinition::Events(events) => events
                .iter()
                .map(|e| model::rsu::RestrictedStockUnitVestingEvent::new(e.date, e.number))
                .collect(),
            RestrictedStockUnitVestingDefinition::Schedule(schedule) => {
                Self::to_events(schedule, &self.grant_value, &self.vesting.commences_on)
            }
        };

        model::rsu::RestrictedStockUnitGrant::new(
            self.name.clone(),
            self.date,
            model::rsu::RestrictedStockUnitValue::new(
                (self.grant_value.grant_price * 100.0) as i32,
                (self.grant_value.total_value * 100.0) as i32,
            ),
            model::rsu::RestrictedStockUnitVestingSchedule::new(self.vesting.commences_on, events),
        )
    }

    fn to_events(
        schedule: &RestrictedStockUnitVestingSchedule,
        value: &RestrictedStockUnitGrantValue,
        commences_on: &NaiveDate,
    ) -> Vec<model::rsu::RestrictedStockUnitVestingEvent> {
        let total_units = (value.total_value / value.grant_price).ceil() as i32;
        let event_length = match schedule.interval {
            RestrictedStockUnitVestingScheduleInterval::Quarterly => 3,
        };
        let count_events = schedule.over.year * 12 / event_length;
        let units_per_event = (total_units as f64) / (count_events as f64);

        let mut date = commences_on.clone();
        let mut events = Vec::new();
        let mut partial_units: f64 = 0.0;

        while events.len() < count_events.try_into().unwrap() {
            date = date.checked_add_months(Months::new(event_length)).unwrap();
            let possible_units = units_per_event + partial_units;
            let units = possible_units.floor() as i32;
            partial_units = possible_units - (units as f64);
            let event = model::rsu::RestrictedStockUnitVestingEvent::new(date, units);
            events.push(event);
        }

        events
    }
}

#[derive(Debug, Deserialize)]
struct RestrictedStockUnitGrantValue {
    /// Unit price of each RSU in the grant, in dollars.
    grant_price: f32,

    /// Total value of the grant, in dollars.
    total_value: f32,
}

#[derive(Debug, Deserialize)]
struct RestrictedStockUnitVesting {
    #[serde(with = "naive_date_format")]
    commences_on: NaiveDate,

    #[serde(flatten)]
    implementation: RestrictedStockUnitVestingDefinition,
}

#[derive(Debug, Deserialize)]
enum RestrictedStockUnitVestingDefinition {
    #[serde(rename = "schedule")]
    Schedule(RestrictedStockUnitVestingSchedule),
    #[serde(rename = "events")]
    Events(RestrictedStockUnitVestingEvents),
}

type RestrictedStockUnitVestingEvents = Vec<RestrictedStockUnitVestingEvent>;

#[derive(Debug, Deserialize)]
struct RestrictedStockUnitVestingSchedule {
    interval: RestrictedStockUnitVestingScheduleInterval,
    over: RestrictedStockUnitVestingScheduleOver,
}

#[derive(Debug, Deserialize)]
enum RestrictedStockUnitVestingScheduleInterval {
    #[serde(rename = "quarterly")]
    Quarterly,
}

#[derive(Debug, Deserialize)]
struct RestrictedStockUnitVestingScheduleOver {
    year: u32,
}

#[derive(Debug, Deserialize)]
struct RestrictedStockUnitVestingEvent {
    #[serde(with = "naive_date_format")]
    date: NaiveDate,
    number: i32,
}

pub fn load_rsu_grants(portfolio_path: &Path) -> Result<Vec<model::rsu::RestrictedStockUnitGrant>> {
    let grants_path = portfolio_path.join("rsu_grants.yaml");

    let contents = fs::read_to_string(&grants_path)
        .map_err(|_| LoadError::FileLoadFailed(grants_path.clone()))?;

    let mut result: Vec<RestrictedStockUnitGrant> = Vec::new();
    for doc in Deserializer::from_str(&contents) {
        let grant = RestrictedStockUnitGrant::deserialize(doc)
            .map_err(LoadError::DeserializationFailed)
            .with_context(|| {
                format!(
                    "Restricted Stock Unit Grant deserialize failed from {:?}",
                    &grants_path
                )
            })?;
        result.push(grant);
    }

    Ok(result.into_iter().map(|g| g.to_model()).collect())
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn test_to_model() {
        let subject = RestrictedStockUnitGrant {
            name: "foo".to_string(),
            date: date(2024, 1, 1),
            grant_value: RestrictedStockUnitGrantValue {
                grant_price: 1.00,
                total_value: 8000.00,
            },
            vesting: RestrictedStockUnitVesting {
                commences_on: date(2024, 1, 1),
                implementation: RestrictedStockUnitVestingDefinition::Schedule(
                    RestrictedStockUnitVestingSchedule {
                        interval: RestrictedStockUnitVestingScheduleInterval::Quarterly,
                        over: RestrictedStockUnitVestingScheduleOver { year: 2 },
                    },
                ),
            },
        };

        assert_eq!(
            model::rsu::RestrictedStockUnitGrant {
                name: "foo".to_string(),
                granted_on: date(2024, 1, 1),
                value: model::rsu::RestrictedStockUnitValue {
                    grant_price_cents: 100,
                    total_value_cents: 8_000_00
                },
                vesting_schedule: model::rsu::RestrictedStockUnitVestingSchedule {
                    commences_on: date(2024, 1, 1),
                    events: vec![
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2024, 4, 1),
                            number: 1000
                        },
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2024, 7, 1),
                            number: 1000
                        },
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2024, 10, 1),
                            number: 1000
                        },
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2025, 1, 1),
                            number: 1000
                        },
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2025, 4, 1),
                            number: 1000
                        },
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2025, 7, 1),
                            number: 1000
                        },
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2025, 10, 1),
                            number: 1000
                        },
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2026, 1, 1),
                            number: 1000
                        },
                    ]
                }
            },
            subject.to_model(),
        );
    }

    #[test]
    fn test_to_model_variable_numbers() {
        let subject = RestrictedStockUnitGrant {
            name: "foo".to_string(),
            date: date(2024, 1, 1),
            grant_value: RestrictedStockUnitGrantValue {
                grant_price: 1.00,
                total_value: 8004.00,
            },
            vesting: RestrictedStockUnitVesting {
                commences_on: date(2024, 1, 1),
                implementation: RestrictedStockUnitVestingDefinition::Schedule(
                    RestrictedStockUnitVestingSchedule {
                        interval: RestrictedStockUnitVestingScheduleInterval::Quarterly,
                        over: RestrictedStockUnitVestingScheduleOver { year: 2 },
                    },
                ),
            },
        };

        assert_eq!(
            model::rsu::RestrictedStockUnitGrant {
                name: "foo".to_string(),
                granted_on: date(2024, 1, 1),
                value: model::rsu::RestrictedStockUnitValue {
                    grant_price_cents: 100,
                    total_value_cents: 8_004_00
                },
                vesting_schedule: model::rsu::RestrictedStockUnitVestingSchedule {
                    commences_on: date(2024, 1, 1),
                    events: vec![
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2024, 4, 1),
                            number: 1000
                        },
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2024, 7, 1),
                            number: 1001
                        },
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2024, 10, 1),
                            number: 1000
                        },
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2025, 1, 1),
                            number: 1001
                        },
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2025, 4, 1),
                            number: 1000
                        },
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2025, 7, 1),
                            number: 1001
                        },
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2025, 10, 1),
                            number: 1000
                        },
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2026, 1, 1),
                            number: 1001
                        },
                    ]
                }
            },
            subject.to_model(),
        );
    }

    #[test]
    fn test_to_model_fractional_shares() {
        let subject = RestrictedStockUnitGrant {
            name: "foo".to_string(),
            date: date(2024, 1, 1),
            grant_value: RestrictedStockUnitGrantValue {
                grant_price: 100.00,
                total_value: 7995.00,
            },
            vesting: RestrictedStockUnitVesting {
                commences_on: date(2024, 1, 1),
                implementation: RestrictedStockUnitVestingDefinition::Schedule(
                    RestrictedStockUnitVestingSchedule {
                        interval: RestrictedStockUnitVestingScheduleInterval::Quarterly,
                        over: RestrictedStockUnitVestingScheduleOver { year: 2 },
                    },
                ),
            },
        };

        assert_eq!(
            model::rsu::RestrictedStockUnitGrant {
                name: "foo".to_string(),
                granted_on: date(2024, 1, 1),
                value: model::rsu::RestrictedStockUnitValue {
                    grant_price_cents: 100_00,
                    total_value_cents: 7_995_00
                },
                vesting_schedule: model::rsu::RestrictedStockUnitVestingSchedule {
                    commences_on: date(2024, 1, 1),
                    events: vec![
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2024, 4, 1),
                            number: 10
                        },
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2024, 7, 1),
                            number: 10
                        },
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2024, 10, 1),
                            number: 10
                        },
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2025, 1, 1),
                            number: 10
                        },
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2025, 4, 1),
                            number: 10
                        },
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2025, 7, 1),
                            number: 10
                        },
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2025, 10, 1),
                            number: 10
                        },
                        model::rsu::RestrictedStockUnitVestingEvent {
                            date: date(2026, 1, 1),
                            number: 10
                        },
                    ]
                }
            },
            subject.to_model(),
        );
    }
}
