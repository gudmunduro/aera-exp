pub mod math;

pub const FLOAT_CMP_THRESHOLD: f64 = 0.00001;

pub fn float_eq(a: f64, b: f64) -> bool {
    float_cmp(a, b, FLOAT_CMP_THRESHOLD)
}

pub fn float_cmp(a: f64, b: f64, threshold: f64) -> bool {
    (a - b).abs() < threshold
}