#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClickInterval {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub milliseconds: u16,
}

impl Default for ClickInterval {
    fn default() -> Self {
        Self {
            hours: 0,
            minutes: 0,
            seconds: 0,
            milliseconds: 200,
        }
    }
}

impl ClickInterval {
    pub fn total_milliseconds(&self) -> u64 {
        (self.hours as u64 * 3600000)
            + (self.minutes as u64 * 60000)
            + (self.seconds as u64 * 1000)
            + (self.milliseconds as u64)
    }
}
