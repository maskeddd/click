use std::time::Duration;

use rand::Rng;
use rand_distr::{Distribution, Gamma, Normal};

#[derive(PartialEq, serde::Deserialize, serde::Serialize, Clone, Copy)]
pub enum IntervalMode {
    Time,
    Cps,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy)]
#[serde(default)]
pub struct TimeInterval {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub milliseconds: u16,
}

impl Default for TimeInterval {
    fn default() -> Self {
        Self {
            hours: 0,
            minutes: 0,
            seconds: 0,
            milliseconds: 200,
        }
    }
}

impl TimeInterval {
    pub fn to_duration(self) -> Duration {
        let total_ms = (self.hours as u64) * 3600 * 1000
            + (self.minutes as u64) * 60 * 1000
            + (self.seconds as u64) * 1000
            + (self.milliseconds as u64);
        Duration::from_millis(total_ms.max(1))
    }
}

pub struct Jitter {
    /// Smoothed offset that creates momentum between clicks
    last_offset: f64,
    click_count: u32,
}

impl Jitter {
    pub fn new() -> Self {
        Self {
            last_offset: 0.0,
            click_count: 0,
        }
    }

    /// Returns the next click delay with humanized timing variations
    ///
    /// `jitter` parameter controls the standard deviation of timing variations in milliseconds
    pub fn next(&mut self, base: Duration, jitter: u16) -> Duration {
        let base_ms = base.as_millis() as f64;
        let mut rng = rand::rng();

        // Normal distribution for continuous small variations
        let normal = Normal::new(0.0, jitter as f64 / 3.0).unwrap();
        let quick_jitter = normal.sample(&mut rng);

        // 3% chance of hesitation spike using gamma distribution
        let hesitation = if rng.random::<f64>() < 0.03 {
            let gamma = Gamma::new(2.0, jitter as f64 * 0.5).unwrap();
            gamma.sample(&mut rng)
        } else {
            0.0
        };

        // Exponential moving average for smooth rhythm changes
        self.last_offset = (self.last_offset * 0.85) + (quick_jitter * 0.15);

        let final_ms = (base_ms + self.last_offset + hesitation).max(1.0);
        self.click_count += 1;
        Duration::from_millis(final_ms.round() as u64)
    }
}
