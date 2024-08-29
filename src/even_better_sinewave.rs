use std::collections::HashMap;

use crate::decimal::DecimalExt;
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use yata::core::{ValueType, Window};
use yata::methods::Cross;
use yata::prelude::Method;

pub type ResultSet = HashMap<String, Decimal>;

pub struct EvenBetterSinewave {
    price: Window<Decimal>,
    hp: Window<Decimal>,
    filt: Window<Decimal>,
    duration: u16,
    upper_cross: Cross,
    lower_cross: Cross,
}

/// Even Better Sinewave Indicator
/// p159 - 164 Cycle Analytics For Traders by John F. Ehlers
/// duration: controls maximum duration of trade when market is in a trend, default 40 bars
/// hold a long position when the indicator is near 1.0
/// hold a short position when the indicator is near -1.0 or close long position
impl EvenBetterSinewave {
    pub fn new(duration: u16) -> Self {
        Self {
            price: Window::new(2, Decimal::ZERO),
            hp: Window::new(2, Decimal::ZERO),
            filt: Window::new(3, Decimal::ZERO),
            duration,
            upper_cross: Cross::default(),
            lower_cross: Cross::default(),
        }
    }

    fn price(&self, i: u16) -> Decimal {
        *self.price.get(i - 1).unwrap_or(&Decimal::ZERO)
    }

    fn hp(&self, i: u16) -> Decimal {
        *self.hp.get(i - 1).unwrap_or(&Decimal::ZERO)
    }

    fn filt(&self, i: u16) -> Decimal {
        *self.filt.get(i - 1).unwrap_or(&Decimal::ZERO)
    }

    pub fn next(&mut self, price: Decimal) -> ResultSet {
        let price1 = self.price(1);
        let filt1 = self.filt(1);
        let filt2 = self.filt(2);
        let hp1 = self.hp(1);

        let alpha1 = (Decimal::ONE
            - (Decimal::TWO * Decimal::PI / Decimal::try_from(self.duration).unwrap()).sin())
            / (Decimal::TWO * Decimal::PI / Decimal::try_from(self.duration).unwrap()).cos();
        let alpha2 = (dec!(-1.414) * Decimal::PI / Decimal::TEN).exp();
        let beta = Decimal::TWO * alpha2 * (dec!(1.414) * Decimal::PI / Decimal::TEN).cos();
        let c2 = beta;
        let c3 = -alpha2 * alpha2;
        let c1 = Decimal::ONE - c2 - c3;

        let hp = dec!(0.5) * (Decimal::ONE + alpha1) * (price - price1) + (alpha1 * hp1);
        let filt = (c1 * ((hp + hp1) / Decimal::TWO)) + (c2 * filt1) + (c3 * filt2);
        let mut signal = (filt + filt1 + filt2) / dec!(3.0);
        let pwr = (filt.powi(2) + filt1.powi(2) + filt2.powi(2)) / dec!(3.0);

        signal /= pwr.sqrt().unwrap();

        self.price.push(price);
        self.filt.push(filt);
        self.hp.push(hp);

        let upper_cross: f64 = self
            .upper_cross
            .next(&(ValueType::try_from(signal).unwrap(), 0.8))
            .analog()
            .into();
        let lower_cross: f64 = self
            .lower_cross
            .next(&(ValueType::try_from(signal).unwrap(), -0.8))
            .analog()
            .into();

        let mut result_set = ResultSet::new();
        result_set.insert("signal".to_string(), signal.to_quantity());
        result_set.insert(
            "upper_cross".to_string(),
            Decimal::try_from(upper_cross).unwrap(),
        );
        result_set.insert(
            "lower_cross".to_string(),
            Decimal::try_from(lower_cross).unwrap(),
        );

        result_set
    }
}
