use std::collections::HashMap;

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use yata::core::Window;
use yata::methods::Cross;
use yata::prelude::Method;

use crate::decimal::DecimalExt;
use crate::digital_signal_processor::*;

pub type ResultSet = HashMap<String, Decimal>;

#[derive(Default)]
pub struct AdaptiveMovingAverage {
    dsp: DigitalSignalProcessor,
    fast_limit: Decimal,
    slow_limit: Decimal,
    phase: Window<Decimal>,
    mama: Window<Decimal>,
    fama: Window<Decimal>,
    cross: Cross,
    trend: Decimal,
    trend_since: Decimal,
}

impl AdaptiveMovingAverage {
    pub fn new(fast_limit: Decimal, slow_limit: Decimal) -> Self {
        Self {
            dsp: DigitalSignalProcessor::new(),
            fast_limit,
            slow_limit,
            phase: Window::new(2, Decimal::ZERO),
            mama: Window::new(2, Decimal::ZERO),
            fama: Window::new(2, Decimal::ZERO),
            ..Default::default()
        }
    }

    fn price(&self, i: u16) -> Decimal {
        self.dsp.price(i)
    }
    fn smooth(&self, i: u16) -> Decimal {
        self.dsp.smooth(i)
    }
    fn detrender(&self, i: u16) -> Decimal {
        self.dsp.detrender(i)
    }
    fn i1(&self, i: u16) -> Decimal {
        self.dsp.i1(i)
    }
    fn i2(&self, i: u16) -> Decimal {
        self.dsp.i2(i)
    }
    fn q1(&self, i: u16) -> Decimal {
        self.dsp.q1(i)
    }
    fn q2(&self, i: u16) -> Decimal {
        self.dsp.q2(i)
    }
    fn re(&self, i: u16) -> Decimal {
        self.dsp.re(i)
    }
    fn im(&self, i: u16) -> Decimal {
        self.dsp.im(i)
    }
    fn period(&self, i: u16) -> Decimal {
        self.dsp.period(i)
    }
    fn smooth_period(&self, i: u16) -> Decimal {
        self.dsp.smooth_period(i)
    }

    fn phase(&self, i: u16) -> Decimal {
        *self.phase.get(i).unwrap_or(&Decimal::ZERO)
    }
    fn mama(&self, i: u16) -> Decimal {
        *self.mama.get(i).unwrap_or(&Decimal::ZERO)
    }
    fn fama(&self, i: u16) -> Decimal {
        *self.fama.get(i).unwrap_or(&Decimal::ZERO)
    }

    pub fn next(&mut self, price: Decimal) -> ResultSet {
        let smooth = calculate_smooth(price, self.price(1), self.price(2), self.price(3));
        let detrender = calculate_detrender(
            smooth,
            self.smooth(2),
            self.smooth(4),
            self.smooth(6),
            self.period(1),
        );
        let q1 = calculate_q1(
            detrender,
            self.detrender(2),
            self.detrender(4),
            self.detrender(6),
            self.period(1),
        );
        let i1 = self.detrender(3);
        let ji = calculate_ji(i1, self.i1(1), self.i1(3), self.i1(5), self.period(1));
        let jq = calculate_jq(q1, self.q1(1), self.q1(3), self.q1(5), self.period(1));
        let i2 = calculate_i2(i1, jq, self.i2(1));
        let q2 = calculate_q2(q1, ji, self.q2(1));
        let re = calculate_re(i2, self.i2(1), q2, self.q2(1), self.re(1));
        let im = calculate_im(i2, q2, self.i2(1), self.q2(1), self.im(1));
        let period = calculate_period(im, re, self.period(1));
        let smooth_period = calculate_smooth_period(period, self.smooth_period(1));

        self.dsp.price.push(price);
        self.dsp.smooth.push(smooth);
        self.dsp.detrender.push(detrender);
        self.dsp.i1.push(i1);
        self.dsp.i2.push(i2);
        self.dsp.q1.push(q1);
        self.dsp.q2.push(q2);
        self.dsp.re.push(re);
        self.dsp.im.push(im);
        self.dsp.period.push(period);
        self.dsp.smooth_period.push(smooth_period);

        let mut phase = Decimal::ZERO;
        if i1 != Decimal::ZERO {
            phase =
                Decimal::try_from((q1.to_f64().unwrap() / i1.to_f64().unwrap()).atan()).unwrap();
        }

        let mut delta_phase = self.phase(1) - phase;
        if delta_phase < Decimal::ONE {
            delta_phase = Decimal::ONE;
        }

        let mut alpha = self.fast_limit / delta_phase;
        if alpha < self.slow_limit {
            alpha = self.slow_limit;
        }
        if alpha > self.fast_limit {
            alpha = self.fast_limit;
        }

        let mama = alpha * price + (Decimal::ONE - alpha) * self.mama(1);
        let fama = dec!(0.5) * alpha * mama + (Decimal::ONE - dec!(0.5) * alpha) * self.fama(1);

        self.phase.push(phase);
        self.mama.push(mama);
        self.fama.push(fama);

        let trend: Decimal = self
            .cross
            .next(&(mama.to_f64().unwrap(), fama.to_f64().unwrap()))
            .analog()
            .into();

        if trend == Decimal::ZERO {
            self.trend_since += Decimal::ONE;
        } else {
            self.trend = trend;
            self.trend_since = Decimal::ZERO;
        }

        let strength = mama
            .checked_sub(fama)
            .unwrap_or(Decimal::ZERO)
            .checked_div(fama)
            .unwrap_or(Decimal::ZERO);

        let mut result_set = ResultSet::new();
        result_set.insert("mama".to_string(), mama.to_quantity());
        result_set.insert("fama".to_string(), fama.to_quantity());
        result_set.insert("trend".to_string(), self.trend);
        result_set.insert("trend_since".to_string(), self.trend_since);
        result_set.insert("strength".to_string(), strength.to_percent());

        result_set
    }
}
