use rust_decimal::{Decimal, RoundingStrategy};

pub const PRECISION_PERCENT: u32 = 3;
pub const PRECISION_MONEY: u32 = 6;
pub const PRECISION_QUANTITY: u32 = 3;

pub trait DecimalExt {
    fn to_percent(&self) -> Decimal;
    fn to_percent_decimal(&self) -> Decimal;
    fn to_money(&self) -> Decimal;
    fn to_quantity(&self) -> Decimal;
    fn is_pos_one(&self) -> bool;
}

impl DecimalExt for Decimal {
    fn to_percent(&self) -> Decimal {
        (self * Decimal::ONE_HUNDRED)
            .round_dp_with_strategy(PRECISION_PERCENT, RoundingStrategy::MidpointAwayFromZero)
    }

    fn to_percent_decimal(&self) -> Decimal {
        self.checked_div(Decimal::ONE_HUNDRED)
            .unwrap_or(Decimal::ZERO)
    }

    fn to_money(&self) -> Decimal {
        self.round_dp_with_strategy(PRECISION_MONEY, RoundingStrategy::MidpointAwayFromZero)
    }

    fn to_quantity(&self) -> Decimal {
        self.round_dp_with_strategy(PRECISION_QUANTITY, RoundingStrategy::MidpointAwayFromZero)
    }

    fn is_pos_one(&self) -> bool {
        *self == Decimal::ONE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_to_percent() {
        let value = dec!(1.2345);
        assert_eq!(value.to_percent(), dec!(123.450));
    }

    #[test]
    fn test_to_percent_decimal() {
        let value = dec!(123.45);
        assert_eq!(value.to_percent_decimal(), dec!(1.2345));
    }

    #[test]
    fn test_to_money() {
        let value = dec!(123.456789);
        assert_eq!(value.to_money(), dec!(123.456789));
    }

    #[test]
    fn test_to_quantity() {
        let value = dec!(123.456789);
        assert_eq!(value.to_quantity(), dec!(123.457));
    }

    #[test]
    fn test_is_pos_one() {
        let value = dec!(1);
        assert!(value.is_pos_one());

        let value = dec!(1.0001);
        assert!(!value.is_pos_one());
    }
}
