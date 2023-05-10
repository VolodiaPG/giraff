use nutype::nutype;
use uom::si::f64::Time;

#[nutype(validate(min = 0, max = 1.0))]
#[derive(Debug, Clone, PartialEq)]
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
