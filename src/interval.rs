use std::time::Duration;

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
