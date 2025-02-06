use std::f64::consts::*;

pub fn probability_density(expected: f64, mean: f64, std: f64) -> f64 {
    (1.0 / (mean * (2.0 * PI).sqrt())) * E.powf((-1.0 / 2.0) * ((expected - mean) / std).powf(2.0))
}