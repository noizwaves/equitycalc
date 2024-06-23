use std::{fs, path::PathBuf};

use chrono::NaiveDate;
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

pub fn load_psp() -> model::psp::PreferredStockPrice {
    let psp_path = "psp.yaml";
    let psp_path = PathBuf::from(psp_path);

    let contents = fs::read_to_string(psp_path).unwrap();

    let mut result: Vec<PreferredStockPrice> = Vec::new();
    for doc in Deserializer::from_str(&contents) {
        let psp = PreferredStockPrice::deserialize(doc).unwrap();
        result.push(psp);
    }

    let valuations = result.into_iter().map(|p| p.to_model()).collect();
    model::psp::PreferredStockPrice::new(valuations)
}
