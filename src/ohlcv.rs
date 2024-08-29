use super::ValueType;
use crate::candles::Source;

pub trait OHLCV: 'static {
    /// Should return an *open* value of the period
    fn open(&self) -> ValueType;

    /// Should return an *highest* value of the period
    fn high(&self) -> ValueType;

    /// Should return an *lowest* value of the period
    fn low(&self) -> ValueType;

    /// Should return an *close* value of the candle
    fn close(&self) -> ValueType;

    /// Should return *volume* value for the period
    fn volume(&self) -> ValueType;

    #[inline]
    fn tp(&self) -> ValueType {
        (self.high() + self.low() + self.close()) / 3.
    }

    #[inline]
    fn hl2(&self) -> ValueType {
        (self.high() + self.low()) * 0.5
    }

    fn ohlc4(&self) -> ValueType {
        (self.high() + self.low() + self.close() + self.open()) * 0.25
    }

    #[inline]
    fn clv(&self) -> ValueType {
        // we need to check division by zero, so we can really just check if `high` is equal to `low` without using any kind of round error checks
        #[allow(clippy::float_cmp)]
        if self.high() == self.low() {
            0.
        } else {
            let twice: ValueType = 2.;
            (twice.mul_add(self.close(), -self.low()) - self.high()) / (self.high() - self.low())
        }
    }

    #[inline]
    fn tr(&self, prev_candle: &dyn OHLCV) -> ValueType {
        self.tr_close(prev_candle.close())
    }

    /// Calculates [True Range](https://en.wikipedia.org/wiki/Average_true_range) over last two candles using `close` price from the previous candle.
    #[inline]
    fn tr_close(&self, prev_close: ValueType) -> ValueType {
        self.high().max(prev_close) - self.low().min(prev_close)
    }

    fn validate(&self) -> bool {
        !(self.close() > self.high() || self.close() < self.low() || self.high() < self.low())
            && self.close() > 0.
            && self.open() > 0.
            && self.high() > 0.
            && self.low() > 0.
            && self.close().is_finite()
            && self.open().is_finite()
            && self.high().is_finite()
            && self.low().is_finite()
            && (self.volume().is_nan() || self.volume() >= 0.0)
    }

    #[inline]
    fn source(&self, source: Source) -> ValueType {
        match source {
            Source::Close => self.close(),
            Source::High => self.high(),
            Source::Low => self.low(),
            Source::TP => self.tp(),
            Source::HL2 => self.hl2(),
            Source::Volume => self.volume(),
            Source::VolumedPrice => self.volumed_price(),
            Source::Open => self.open(),
        }
    }

    /// Volumed price
    ///
    /// Same as [`OHLCV::tp()`] * [`OHLCV::volume()`]
    fn volumed_price(&self) -> ValueType {
        self.tp() * self.volume()
    }

    /// Checks if candle is "rising": it's close value greater than open value
    fn is_rising(&self) -> bool {
        self.close() > self.open()
    }

    /// Checks if candle is "falling": it's close value smaller than open value
    fn is_falling(&self) -> bool {
        self.close() < self.open()
    }
}

impl OHLCV for (ValueType, ValueType, ValueType, ValueType, ValueType) {
    #[inline]
    fn open(&self) -> ValueType {
        self.0
    }

    #[inline]
    fn high(&self) -> ValueType {
        self.1
    }

    #[inline]
    fn low(&self) -> ValueType {
        self.2
    }

    #[inline]
    fn close(&self) -> ValueType {
        self.3
    }

    #[inline]
    fn volume(&self) -> ValueType {
        self.4
    }
}

impl OHLCV for [ValueType; 5] {
    #[inline]
    fn open(&self) -> ValueType {
        self[0]
    }

    #[inline]
    fn high(&self) -> ValueType {
        self[1]
    }

    #[inline]
    fn low(&self) -> ValueType {
        self[2]
    }

    #[inline]
    fn close(&self) -> ValueType {
        self[3]
    }

    #[inline]
    fn volume(&self) -> ValueType {
        self[4]
    }
}
