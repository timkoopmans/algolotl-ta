use std::collections::HashMap;

use crate::decimal::DecimalExt;
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use yata::core::{ValueType, Window};
use yata::methods::Cross;
use yata::prelude::Method;

pub type ResultSet = HashMap<String, Decimal>;

#[derive(Default)]
pub struct SuperSmootherFilter {
    price: Window<Decimal>,
    filter: Window<Decimal>,
    cross: Cross,
    trend: Decimal,
    trend_since: Decimal,
}

impl SuperSmootherFilter {
    pub fn new() -> Self {
        SuperSmootherFilter {
            price: Window::new(2, Decimal::ZERO),
            filter: Window::new(3, Decimal::ZERO),
            ..Default::default()
        }
    }

    fn price(&self, i: u16) -> Decimal {
        *self.price.get(i - 1).unwrap_or(&Decimal::ZERO)
    }

    fn filter(&self, i: u16) -> Decimal {
        *self.filter.get(i - 1).unwrap_or(&Decimal::ZERO)
    }

    pub fn next(&mut self, price: Decimal) -> ResultSet {
        let a1 = (dec!(-1.414) * Decimal::PI / Decimal::TEN).exp();
        let b1 =
            Decimal::TWO * a1 * (dec!(1.414) * Decimal::TWO * Decimal::PI / Decimal::TEN).cos();
        let c2 = b1;
        let c3 = -a1 * a1;
        let c1 = Decimal::ONE - c2 - c3;
        let filter =
            c1 * (price + self.price(1)) / Decimal::TWO + c2 * self.filter(1) + c3 * self.filter(2);

        let trigger = self.filter(2);

        self.price.push(price);
        self.filter.push(filter);

        let cross: Decimal = self
            .cross
            .next(&(
                ValueType::try_from(filter).unwrap(),
                ValueType::try_from(trigger).unwrap(),
            ))
            .analog()
            .into();

        if cross == Decimal::ZERO {
            self.trend_since += Decimal::ONE;
        } else {
            self.trend = cross;
            self.trend_since = Decimal::ZERO;
        }

        let strength = filter
            .checked_sub(trigger)
            .unwrap_or(Decimal::ZERO)
            .checked_div(trigger)
            .unwrap_or(Decimal::ZERO);

        let mut result_set = ResultSet::new();
        result_set.insert("filter".to_string(), filter.to_quantity());
        result_set.insert("trigger".to_string(), trigger.to_quantity());
        result_set.insert("cross".to_string(), cross);
        result_set.insert("trend".to_string(), self.trend);
        result_set.insert("trend_since".to_string(), self.trend_since);
        result_set.insert("strength".to_string(), strength.to_percent());

        result_set
    }
}
