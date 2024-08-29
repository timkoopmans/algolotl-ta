use crate::window::Window;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Default)]
pub struct DigitalSignalProcessor {
    pub price: Window<Decimal>,
    pub smooth: Window<Decimal>,
    pub detrender: Window<Decimal>,
    pub i1: Window<Decimal>,
    pub q1: Window<Decimal>,
    pub i2: Window<Decimal>,
    pub q2: Window<Decimal>,
    pub q3: Window<Decimal>,
    pub re: Window<Decimal>,
    pub im: Window<Decimal>,
    pub period: Window<Decimal>,
    pub smooth_period: Window<Decimal>,
}

impl DigitalSignalProcessor {
    pub fn new() -> Self {
        Self {
            price: Window::new(4, Decimal::ZERO),
            smooth: Window::new(7, Decimal::ZERO),
            detrender: Window::new(7, Decimal::ZERO),
            i1: Window::new(6, Decimal::ZERO),
            q1: Window::new(6, Decimal::ZERO),
            i2: Window::new(2, Decimal::ZERO),
            q2: Window::new(2, Decimal::ZERO),
            q3: Window::new(2, Decimal::ZERO),
            re: Window::new(2, Decimal::ZERO),
            im: Window::new(2, Decimal::ZERO),
            period: Window::new(2, Decimal::ZERO),
            smooth_period: Window::new(2, Decimal::ZERO),
        }
    }

    pub fn price(&self, i: u8) -> Decimal {
        *self.price.get(i - 1).unwrap_or(&Decimal::ZERO)
    }
    pub fn period(&self, i: u8) -> Decimal {
        *self.period.get(i - 1).unwrap_or(&Decimal::ZERO)
    }
    pub fn smooth(&self, i: u8) -> Decimal {
        *self.smooth.get(i - 1).unwrap_or(&Decimal::ZERO)
    }
    pub fn smooth_period(&self, i: u8) -> Decimal {
        *self.smooth_period.get(i - 1).unwrap_or(&Decimal::ZERO)
    }
    pub fn detrender(&self, i: u8) -> Decimal {
        *self.detrender.get(i - 1).unwrap_or(&Decimal::ZERO)
    }
    pub fn re(&self, i: u8) -> Decimal {
        *self.re.get(i - 1).unwrap_or(&Decimal::ZERO)
    }
    pub fn im(&self, i: u8) -> Decimal {
        *self.im.get(i - 1).unwrap_or(&Decimal::ZERO)
    }
    pub fn i1(&self, i: u8) -> Decimal {
        *self.i1.get(i - 1).unwrap_or(&Decimal::ZERO)
    }
    pub fn i2(&self, i: u8) -> Decimal {
        *self.i2.get(i - 1).unwrap_or(&Decimal::ZERO)
    }
    pub fn q1(&self, i: u8) -> Decimal {
        *self.q1.get(i - 1).unwrap_or(&Decimal::ZERO)
    }
    pub fn q2(&self, i: u8) -> Decimal {
        *self.q2.get(i - 1).unwrap_or(&Decimal::ZERO)
    }
    pub fn q3(&self, i: u8) -> Decimal {
        *self.q3.get(i - 1).unwrap_or(&Decimal::ZERO)
    }
}

pub fn calculate_smooth(
    price: Decimal,
    price_1: Decimal,
    price_2: Decimal,
    price_3: Decimal,
) -> Decimal {
    (dec!(4.0) * price + dec!(3.0) * price_1 + dec!(2.0) * price_2 + price_3) / dec!(10.0)
}

pub fn calculate_detrender(
    smooth: Decimal,
    smooth_2: Decimal,
    smooth_4: Decimal,
    smooth_6: Decimal,
    period_1: Decimal,
) -> Decimal {
    (dec!(0.0962) * smooth + dec!(0.5769) * smooth_2
        - dec!(0.5769) * smooth_4
        - dec!(0.0962) * smooth_6)
        * (dec!(0.075) * period_1 + dec!(0.54))
}

pub fn calculate_q1(
    detrender: Decimal,
    detrender_2: Decimal,
    detrender_4: Decimal,
    detrender_6: Decimal,
    period_1: Decimal,
) -> Decimal {
    (dec!(0.0962) * detrender + dec!(0.5769) * detrender_2
        - dec!(0.5769) * detrender_4
        - dec!(0.0962) * detrender_6)
        * (dec!(0.075) * period_1 + dec!(0.54))
}

pub fn calculate_ji(
    i1: Decimal,
    i1_1: Decimal,
    i1_3: Decimal,
    i1_5: Decimal,
    period_1: Decimal,
) -> Decimal {
    (dec!(0.0962) * i1 + dec!(0.5769) * i1_1 - dec!(0.5769) * i1_3 - dec!(0.0962) * i1_5)
        * (dec!(0.075) * period_1 + dec!(0.54))
}

pub fn calculate_jq(
    q1: Decimal,
    q1_1: Decimal,
    q1_3: Decimal,
    q1_5: Decimal,
    period_1: Decimal,
) -> Decimal {
    (dec!(0.0962) * q1 + dec!(0.5769) * q1_1 - dec!(0.5769) * q1_3 - dec!(0.0962) * q1_5)
        * (dec!(0.075) * period_1 + dec!(0.54))
}

pub fn calculate_i2(i1: Decimal, jq: Decimal, i2_1: Decimal) -> Decimal {
    dec!(0.2) * (i1 - jq) + dec!(0.8) * i2_1
}

pub fn calculate_q2(q1: Decimal, ji: Decimal, q2_1: Decimal) -> Decimal {
    dec!(0.2) * (q1 + ji) + dec!(0.8) * q2_1
}

pub fn calculate_re(
    i2: Decimal,
    i2_1: Decimal,
    q2: Decimal,
    q2_1: Decimal,
    re_1: Decimal,
) -> Decimal {
    dec!(0.2) * (i2 * i2_1 + q2 * q2_1) + dec!(0.8) * re_1
}

pub fn calculate_im(
    i2: Decimal,
    q2: Decimal,
    i2_1: Decimal,
    q2_1: Decimal,
    im_1: Decimal,
) -> Decimal {
    dec!(0.2) * (i2 * q2_1 - q2 * i2_1) + dec!(0.8) * im_1
}

pub fn calculate_period(im: Decimal, re: Decimal, period_1: Decimal) -> Decimal {
    let mut period = if im != Decimal::ZERO && re != Decimal::ZERO {
        Decimal::try_from(360.0 / (im.to_f64().unwrap() / re.to_f64().unwrap()).atan()).unwrap()
    } else {
        Decimal::ZERO
    };
    if period > dec!(1.5) * period_1 {
        period = dec!(1.5) * period_1;
    }
    if period < dec!(0.67) * period_1 {
        period = dec!(0.67) * period_1;
    }
    if period < dec!(6.0) {
        period = dec!(6.0);
    }
    if period > dec!(50.0) {
        period = dec!(50.0);
    }
    dec!(0.2) * period + dec!(0.8) * period_1
}

pub fn calculate_smooth_period(period: Decimal, smooth_period_1: Decimal) -> Decimal {
    dec!(0.33) * period + dec!(0.67) * smooth_period_1
}

pub fn calculate_q3(smooth: Decimal, smooth_2: Decimal, smooth_period: Decimal) -> Decimal {
    dec!(0.5) * (smooth - smooth_2) * (dec!(0.1759) * smooth_period + dec!(0.4607))
}

pub fn calculate_snr(signal: Decimal, noise: Decimal, snr_1: Decimal) -> Decimal {
    if noise != Decimal::ZERO && signal != Decimal::ZERO {
        Decimal::try_from(
            0.33 * (10.0 * (signal.to_f64().unwrap() / noise.to_f64().unwrap()).ln()
                / 10.0_f64.log10())
                + 0.67 * snr_1.to_f64().unwrap(),
        )
        .unwrap()
    } else {
        Decimal::ZERO
    }
}
