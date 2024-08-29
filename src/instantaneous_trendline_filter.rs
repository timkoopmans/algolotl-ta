use crate::decimal::DecimalExt;
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use std::collections::HashMap;
use yata::core::{ValueType, Window};
use yata::methods::Cross;
use yata::prelude::Method;

pub type ResultSet = HashMap<String, Decimal>;

#[derive(Default)]
pub struct InstantaneousTrendlineFilter {
    price: Window<Decimal>,
    filter: Window<Decimal>,
    current_bar: usize,
    cross: Cross,
    trend: Decimal,
    trend_since: Decimal,
}

impl InstantaneousTrendlineFilter {
    pub fn new() -> Self {
        InstantaneousTrendlineFilter {
            price: Window::new(2, Decimal::ZERO),
            filter: Window::new(3, Decimal::ZERO),
            current_bar: 0,
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
        let alpha: Decimal = dec!(0.07);
        self.current_bar += 1;

        let filter = if self.current_bar < 7 {
            (price + (Decimal::TWO * self.price(1)) + self.price(2)) / dec!(4.0)
        } else {
            ((alpha - (alpha.powi(2) / dec!(4.0))) * price)
                + (dec!(0.5) * alpha.powi(2) * self.price(1))
                - ((alpha - (dec!(0.75) * alpha.powi(2))) * self.price(2))
                + (Decimal::TWO * (Decimal::ONE - alpha) * self.filter(1))
                - ((Decimal::ONE - alpha).powi(2) * self.filter(2))
        };

        let trigger = (Decimal::TWO * filter) - self.filter(2);

        self.price.push(price);
        self.filter.push(filter);

        let cross: Decimal = self
            .cross
            .next(&(
                ValueType::try_from(trigger).unwrap(),
                ValueType::try_from(filter).unwrap(),
            ))
            .analog()
            .into();

        if cross == Decimal::ZERO {
            self.trend_since += Decimal::ONE;
        } else {
            self.trend = cross;
            self.trend_since = Decimal::ZERO;
        }

        let strength = trigger
            .checked_sub(filter)
            .unwrap_or(Decimal::ZERO)
            .checked_div(filter)
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
