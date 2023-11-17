use nutype::nutype;
use uom::si::f64::Time;

#[nutype(
    derive(Debug, Clone, PartialEq),
    validate(greater_or_equal = 0, less_or_equal = 1.0)
)]
pub struct Alpha(f64);

#[derive(Debug, Clone)]
pub struct ExponentialMovingAverage {
    alpha:       Alpha,
    current_ema: Time,
}

impl ExponentialMovingAverage {
    pub fn new(alpha: Alpha, initial_ema: Time) -> Self {
        Self { alpha, current_ema: initial_ema }
    }

    pub fn update(&mut self, value: Time) {
        let alpha = self.alpha.clone().into_inner();
        self.current_ema = alpha * value + (1.0 - alpha) * self.current_ema;
    }

    pub fn get(&self) -> Time { self.current_ema }
}
