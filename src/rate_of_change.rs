use std::collections::HashMap;

use crate::decimal::DecimalExt;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use yata::core::ValueType;
use yata::prelude::Candle;

pub struct RateOfChange {
    period: usize,
}

pub type ResultSet = HashMap<String, Decimal>;

impl RateOfChange {
    pub fn new(period: usize) -> Self {
        Self { period }
    }

    pub fn next(&mut self, i: usize, candle: &Candle, candles: &[Candle]) -> ResultSet {
        let mut strength = Decimal::ZERO;

        for possible_candle in &candles[i + 1..i + 1 + self.period.min(candles.len() - i - 1)] {
            let roc: ValueType = (possible_candle.close - candle.close) / candle.close;
            let rocd = Decimal::from_f64(roc.to_f64().unwrap()).unwrap();

            if rocd.abs() > strength.abs() {
                strength = rocd;
            }
        }

        let mut result_set = ResultSet::new();
        result_set.insert("strength".to_string(), strength.to_quantity());

        result_set
    }
}
