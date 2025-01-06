use std::f64::consts::PI;

pub struct Wavetable {
    samples: Vec<f64>,
}

pub struct WavetableIter<'a> {
    pub frequency: f64,
    index: f64,
    sample_rate: f64,
    wavetable: &'a Wavetable,
}

impl Wavetable {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);

        Self {
            samples: Vec::with_capacity(capacity),
        }
    }

    pub fn fill<F>(&mut self, waveform: F)
    where
        F: Fn(f64) -> f64 + 'static,
    {
        for i in 0..self.samples.capacity() {
            let phase = 2.0 * PI * i as f64 / self.samples.capacity() as f64;
            let sample = waveform(phase);
            self.samples.push(sample);
        }
    }

    pub fn iter(&self, frequency: f64, sample_rate: f64) -> WavetableIter {
        WavetableIter {
            frequency,
            index: 0.0,
            sample_rate,
            wavetable: &self,
        }
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }
}

impl<'a> Iterator for WavetableIter<'a> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.wavetable.samples[self.index.floor() as usize];

        self.index += self.frequency * self.wavetable.len() as f64 / self.sample_rate as f64;
        self.index %= self.wavetable.len() as f64;

        Some(sample)
    }
}

#[cfg(test)]
mod tests {
    use crate::waveform;
    use std::f64::EPSILON;

    use super::*;

    #[test]
    fn sine() {
        let mut sine_table = Wavetable::new(64);

        sine_table.fill(waveform::sine);

        assert!(sine_table.len() == 64);
        assert!((0.0 - sine_table.samples[0]).abs() < EPSILON);
        assert!((1.0 - sine_table.samples[16]).abs() < EPSILON);
        assert!((0.0 - sine_table.samples[32]).abs() < EPSILON);
        assert!((-1.0 - sine_table.samples[48]).abs() < EPSILON);
    }

    #[test]
    fn sawtooth() {
        let mut sawtooth_table = Wavetable::new(64);

        sawtooth_table.fill(waveform::sawtooth);

        assert!(sawtooth_table.len() == 64);
        assert!((0.0 - sawtooth_table.samples[0]).abs() < EPSILON);
        assert!((1.0 - sawtooth_table.samples[31]).abs() < 0.05);
        assert!((-1.0 - sawtooth_table.samples[32]).abs() < EPSILON);
        assert!((0.0 - sawtooth_table.samples[63]).abs() < 0.05);
    }

    #[test]
    fn iteration() {
        let frequency = 440.0;
        let length = 64 as usize;
        let sample_rate = 44_000.0;

        let mut sine_table = Wavetable::new(length);

        sine_table.fill(waveform::sine);

        let mut iter = sine_table.iter(frequency, sample_rate);

        let index_increment = frequency * length as f64 / sample_rate;

        let a = 0;
        let b = (index_increment % length as f64).floor() as usize;
        let c = (index_increment + (index_increment % length as f64)).floor() as usize;

        assert!(iter.next().unwrap() == sine_table.samples[a]);
        assert!(iter.next().unwrap() == sine_table.samples[b]);
        assert!(iter.next().unwrap() == sine_table.samples[c]);
    }
}
