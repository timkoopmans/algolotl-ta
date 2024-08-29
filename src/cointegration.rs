use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::collections::HashMap;
use tradestats::metrics::{
    engle_granger_cointegration_test, pearson_correlation_coefficient, spread_dynamic,
    spread_standard,
};
use yata::core::{PeriodType, Window};

pub struct Cointegration {
    pub x: Window<f64>,
    pub y: Window<f64>,
}

pub type ResultSet = HashMap<String, Decimal>;

impl Cointegration {
    pub fn new(period: usize) -> Self {
        Self {
            x: Window::new(period as PeriodType, 0.),
            y: Window::new(period as PeriodType, 0.),
        }
    }

    pub fn next(&mut self, x: f64, y: f64) -> ResultSet {
        let mut result = HashMap::new();

        self.x.push(x);
        self.y.push(y);

        let x: Vec<f64> = self.x.iter().copied().collect();
        let y: Vec<f64> = self.y.iter().copied().collect();

        let spread_std: Vec<f64> = spread_standard(&x, &y).unwrap();
        result.insert(
            "spread_std".to_string(),
            Decimal::from_f64(spread_std.last().unwrap().clone()).unwrap(),
        );

        let spread_dyn: Vec<f64> = spread_dynamic(&x, &y).unwrap();
        result.insert(
            "spread_dyn".to_string(),
            Decimal::from_f64(spread_dyn.last().unwrap().clone()).unwrap(),
        );

        let e_coint = engle_granger_cointegration_test(&x, &y);
        match e_coint {
            Ok(coint) => {
                result.insert(
                    "engle_t_stat".to_string(),
                    Decimal::from_f64(coint.test_statistic).unwrap_or(Decimal::ZERO),
                );
                result.insert(
                    "engle_p_value".to_string(),
                    Decimal::from_f64(coint.p_value).unwrap_or(Decimal::ZERO),
                );
                result.insert(
                    "is_coint".to_string(),
                    Decimal::from_f64(f64::from(coint.is_coint)).unwrap(),
                );
            }
            Err(_) => return result,
        };

        let p_coint = pearson_correlation_coefficient(&x, &y);
        match p_coint {
            Ok(pearson) => {
                result.insert("pearson".to_string(), Decimal::from_f64(pearson).unwrap());
            }
            Err(_) => return result,
        };

        let btc_mean = x.iter().sum::<f64>() / x.len() as f64;
        let alt_mean = y.iter().sum::<f64>() / y.len() as f64;

        let mut covariance = 0.0;
        let mut btc_variance = 0.0;
        let mut alt_variance = 0.0;

        for i in 0..x.len() {
            let btc_diff = x[i] - btc_mean;
            let alt_diff = y[i] - alt_mean;
            covariance += btc_diff * alt_diff;
            btc_variance += btc_diff * btc_diff;
            alt_variance += alt_diff * alt_diff;
        }

        let correlation = covariance / (btc_variance.sqrt() * alt_variance.sqrt());

        result.insert(
            "correlation".to_string(),
            Decimal::from_f64(correlation).unwrap(),
        );

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cointegration_next() {
        let mut cointegration = Cointegration::new();

        let series_1 = vec![
            0.5638, 0.5519, 0.557, 0.5571, 0.5577, 0.5547, 0.5581, 0.5582, 0.5577, 0.5617, 0.5656,
            0.5613, 0.5589, 0.5606, 0.561, 0.565, 0.567, 0.5662, 0.5749, 0.5719, 0.5747, 0.563,
            0.5568,
        ];
        let series_2 = vec![
            0.06683, 0.0662, 0.06668, 0.06673, 0.06675, 0.06681, 0.06686, 0.06693, 0.06679,
            0.06695, 0.0667, 0.06699, 0.06696, 0.06714, 0.06683, 0.06746, 0.06775, 0.06771,
            0.06837, 0.06787, 0.06798, 0.0669, 0.06649,
        ];

        for (x, y) in series_1.iter().zip(series_2.iter()) {
            let result = cointegration.next(*x, *y);
            println!("{:?}", result);
        }
    }
}
