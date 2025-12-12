#[derive(Debug, Clone, Copy)]
pub enum IntervalField {
    Hours,
    Minutes,
    Seconds,
    Milliseconds,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ClickInterval {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub milliseconds: u16,
}

impl ClickInterval {
    pub fn total_milliseconds(&self) -> u64 {
        (self.hours as u64 * 3600000)
            + (self.minutes as u64 * 60000)
            + (self.seconds as u64 * 1000)
            + (self.milliseconds as u64)
    }
}
