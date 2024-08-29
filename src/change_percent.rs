use crate::decimal::DecimalExt;
use rust_decimal::Decimal;

pub struct ChangePercent {
    prev: Option<Decimal>,
}

impl ChangePercent {
    pub fn new() -> Self {
        Self {
            prev: None,
        }
    }

    pub fn next(&mut self, curr: Decimal) -> Decimal {
        let mut prev = self.prev.unwrap_or(curr);

        if prev.is_zero() {
            prev = curr;
        }

        let ratio = curr / prev;
        let chg_pct = (ratio - Decimal::ONE).to_percent();

        self.prev = Some(curr);

        chg_pct
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_change_percent() {
        let mut change_percent = ChangePercent::new();

        let curr1 = dec!(100);
        let curr2 = dec!(110);
        let curr3 = dec!(121);

        let chg_pct1 = change_percent.next(curr1);
        assert_eq!(chg_pct1, dec!(0)); // No change since it's the first value

        let chg_pct2 = change_percent.next(curr2);
        assert_eq!(chg_pct2, dec!(10)); // 10% increase

        let chg_pct3 = change_percent.next(curr3);
        assert_eq!(chg_pct3, dec!(10)); // Another 10% increase
    }
}
