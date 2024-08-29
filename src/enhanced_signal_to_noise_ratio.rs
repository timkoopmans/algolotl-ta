use std::collections::HashMap;

use crate::decimal::DecimalExt;
use crate::digital_signal_processor::*;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use yata::core::Window;

pub type ResultSet = HashMap<String, Decimal>;

pub struct EnhancedSignalToNoiseRatio {
    dsp: DigitalSignalProcessor,
    noise: Window<Decimal>,
    snr: Window<Decimal>,
}

/// Enhanced Signal to Noise Ratio
/// p87 - 88 Rocket Science for Traders by John F. Ehlers
/// A high SNR indicates a strong signal (potential trading opportunity) with
/// little noise (random price fluctuations), while a low SNR indicates a weak
/// signal with a lot of noise.
/// Total lag for this indicator is 4 bars.
/// Cycle mode trading should be avoided when the SNR is below 6dB
impl EnhancedSignalToNoiseRatio {
    pub fn new() -> Self {
        Self {
            dsp: DigitalSignalProcessor::new(),
            noise: Window::new(2, Decimal::ZERO),
            snr: Window::new(2, Decimal::ZERO),
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
    fn q3(&self, i: u16) -> Decimal {
        self.dsp.q3(i)
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

    fn noise(&self, i: u16) -> Decimal {
        *self.noise.get(i - 1).unwrap_or(&Decimal::ZERO)
    }
    fn snr(&self, i: u16) -> Decimal {
        *self.snr.get(i - 1).unwrap_or(&Decimal::ZERO)
    }

    pub fn next(&mut self, price: Decimal, high: Decimal, low: Decimal) -> ResultSet {
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
        let q3 = calculate_q3(smooth, self.smooth(2), smooth_period);

        self.dsp.price.push(price);
        self.dsp.smooth.push(smooth);
        self.dsp.detrender.push(detrender);
        self.dsp.i1.push(i1);
        self.dsp.i2.push(i2);
        self.dsp.q1.push(q1);
        self.dsp.q2.push(q2);
        self.dsp.q3.push(q3);
        self.dsp.re.push(re);
        self.dsp.im.push(im);
        self.dsp.period.push(period);
        self.dsp.smooth_period.push(smooth_period);

        let mut i3 = q3;
        for i in 1..(smooth_period.to_f64().unwrap() / 2.0).ceil() as u16 {
            i3 += self.q3(i);
        }
        i3 = dec!(1.57) * i3 / (smooth_period / Decimal::TWO);

        let signal = i3 * i3 + q3 * q3;
        let noise =
            dec!(0.1) * (high - low) * (high - low) * dec!(0.25) + dec!(0.9) * self.noise(1);
        let snr = calculate_snr(signal, noise, self.snr(1));

        let mut result_set = ResultSet::new();
        result_set.insert("snr".to_string(), snr.to_quantity());

        result_set
    }
}
