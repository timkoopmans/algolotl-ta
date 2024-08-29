use std::collections::HashMap;

use crate::decimal::DecimalExt;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use yata::core::{ValueType, Window};
use yata::methods::{Cross, HighestIndex, LowestIndex, SMA};
use yata::prelude::Method;

pub type ResultSet = HashMap<String, Decimal>;

pub struct EmpiricalModeDecomposition {
    delta: Decimal,
    fraction: Decimal,
    period: u16,
    price: Window<Decimal>,
    bp: Window<Decimal>,
    bp_sma: SMA,
    peak: Window<Decimal>,
    peak_sma: SMA,
    valley: Window<Decimal>,
    valley_sma: SMA,
    cross_hi: Cross,
    cross_lo: Cross,
    highest: HighestIndex,
    lowest: LowestIndex,
    trend: i64,
    trend_since: i64,
}

/// Empirical Mode Decomposition
/// https://www.mesasoftware.com/papers/EmpiricalModeDecomposition.pdf
///
/// delta: controls the amount of damping, default 0.5
/// fraction: controls the amount of smoothing, default 0.1
/// period: the period of the cycle, default 20
/// returns: (mean_bp, mean_peak, mean_valley)
/// mean_bp: the mean of the band pass filter
/// mean_peak: the mean of the peak
/// mean_valley: the mean of the valley
///
/// The cycle component is extracted by bandpass filtering the data.
/// The trend component is extracted by averaging the bandpass filtered data over
/// the most recent two cycle periods (to get smoothing without too much lag).
/// If the trend is above the upper threshold the market is in an uptrend.
/// If the trend is below the lower threshold the market is in a downtrend.
/// When the trend falls between the two threshold levels the market is in a cycle mode.
impl EmpiricalModeDecomposition {
    pub fn new(delta: Decimal, fraction: Decimal, period: u16) -> Self {
        Self {
            delta,
            fraction,
            period,
            price: Window::new(2 * period, Decimal::ZERO),
            bp: Window::new(3, Decimal::ZERO),
            bp_sma: SMA::new(2 * period, &0.0).unwrap(),
            peak: Window::new(2, Decimal::ZERO),
            peak_sma: SMA::new(50, &0.0).unwrap(),
            valley: Window::new(2, Decimal::ZERO),
            valley_sma: SMA::new(50, &0.0).unwrap(),
            cross_hi: Cross::default(),
            cross_lo: Cross::default(),
            highest: HighestIndex::new(48, &0.0).unwrap(),
            lowest: LowestIndex::new(48, &0.0).unwrap(),
            trend: 0,
            trend_since: 0,
        }
    }

    fn price(&self, i: u16) -> Decimal {
        *self.price.get(i - 1).unwrap_or(&Decimal::ZERO)
    }

    fn bp(&self, i: u16) -> Decimal {
        *self.bp.get(i - 1).unwrap_or(&Decimal::ZERO)
    }

    fn peak(&self, i: u16) -> Decimal {
        *self.peak.get(i - 1).unwrap_or(&Decimal::ZERO)
    }

    fn valley(&self, i: u16) -> Decimal {
        *self.valley.get(i - 1).unwrap_or(&Decimal::ZERO)
    }

    pub fn next(&mut self, price: Decimal) -> ResultSet {
        let price2 = self.price(2);
        let bp1 = self.bp(1);
        let bp2 = self.bp(2);
        let peak1 = self.peak(1);
        let valley1 = self.valley(1);

        let beta = (Decimal::TWO * Decimal::PI / Decimal::try_from(self.period).unwrap()).cos();
        let gamma = Decimal::ONE
            / (dec!(4.0) * Decimal::PI * self.delta / Decimal::try_from(self.period).unwrap())
                .cos();
        let alpha = gamma - (gamma.powi(2) - Decimal::ONE).sqrt().unwrap();

        let bp = dec!(0.5) * (Decimal::ONE - alpha) * (price - price2)
            + beta * (Decimal::ONE + alpha) * bp1
            - alpha * bp2;

        let mean_bp = self.bp_sma.next(&bp.to_f64().unwrap());

        let mut peak = peak1;
        let mut valley = valley1;

        if bp1 > bp && bp1 > bp2 {
            peak = bp1;
        }

        if bp1 < bp && bp1 < bp2 {
            valley = bp1;
        }

        let mean_peak = self.fraction
            * Decimal::try_from(self.peak_sma.next(&ValueType::try_from(peak).unwrap())).unwrap();
        let mean_valley = self.fraction
            * Decimal::try_from(self.valley_sma.next(&ValueType::try_from(valley).unwrap()))
                .unwrap();

        self.bp.push(bp);
        self.price.push(price);
        self.peak.push(peak);
        self.valley.push(valley);

        let upper_cross: i64 = self
            .cross_hi
            .next(&(mean_bp, ValueType::try_from(mean_peak).unwrap()))
            .analog()
            .into();
        let lower_cross: i64 = self
            .cross_lo
            .next(&(mean_bp, ValueType::try_from(mean_valley).unwrap()))
            .analog()
            .into();

        let highest_high: i64 = (self.highest.next(&mean_bp) == 1).into();
        let lowest_low: i64 = (self.lowest.next(&mean_bp) == 1).into();

        if lower_cross == -1 {
            // downtrend
            self.trend = -1;
            self.trend_since = 0;
        } else if upper_cross == 1 {
            // uptrend
            self.trend = 1;
            self.trend_since = 0;
        } else if upper_cross == -1 || lower_cross == 1 {
            // cycle
            self.trend = 0;
            self.trend_since = 0;
        } else {
            self.trend_since += 1;
        }

        let mut result_set = ResultSet::new();
        result_set.insert(
            "mean".to_string(),
            Decimal::try_from(mean_bp).unwrap().to_quantity(),
        );
        result_set.insert("upper".to_string(), mean_peak.to_quantity());
        result_set.insert("upper_cross".to_string(), Decimal::from(upper_cross));
        result_set.insert("lower".to_string(), mean_valley.to_quantity());
        result_set.insert("lower_cross".to_string(), Decimal::from(lower_cross));
        result_set.insert("highest_high".to_string(), Decimal::from(highest_high));
        result_set.insert("lowest_low".to_string(), Decimal::from(lowest_low));
        result_set.insert("trend".to_string(), Decimal::from(self.trend));
        result_set.insert("trend_since".to_string(), Decimal::from(self.trend_since));

        result_set
    }
}
