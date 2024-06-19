use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct PreferredStockPriceValuation {
    date: NaiveDate,
    value_cents: i32,
}

impl PreferredStockPriceValuation {
    pub fn new(date: NaiveDate, value_cents: i32) -> PreferredStockPriceValuation {
        PreferredStockPriceValuation { date, value_cents }
    }
}

#[derive(Debug)]
pub struct PreferredStockPrice {
    values: Vec<PreferredStockPriceValuation>,
}

impl PreferredStockPrice {
    pub fn new(values: Vec<PreferredStockPriceValuation>) -> PreferredStockPrice {
        let mut sorted = values.clone();
        sorted.sort_by(|a, b| a.date.cmp(&b.date));

        PreferredStockPrice { values: sorted }
    }

    pub fn value_on(&self, date: &NaiveDate) -> i32 {
        // The first valuation after `date`
        self.values
            .iter()
            .take_while(|valuation| &valuation.date <= date)
            .last()
            .expect(&format!("No valuation found for {date}"))
            .value_cents
    }
}
