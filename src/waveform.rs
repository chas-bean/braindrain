use std::f64::consts::PI;

pub fn sine(phase: f64) -> f64 {
    phase.sin()
}

pub fn sawtooth(phase: f64) -> f64 {
    ((phase + PI) / PI) % 2.0 - 1.0
}
