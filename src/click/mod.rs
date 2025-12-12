use anyhow::Result;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::PlatformClicker;
#[cfg(target_os = "macos")]
pub use macos::PlatformClicker;
#[cfg(target_os = "windows")]
pub use windows::PlatformClicker;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClickType {
    Single,
    Hold,
}

pub trait ClickerBackend: Send {
    fn click(&mut self, button: MouseButton, click_type: ClickType) -> Result<()>;
}

pub struct Clicker {
    backend: PlatformClicker,
    delay_ms: u64,
}

impl Clicker {
    pub fn new(delay_ms: u64) -> Result<Self> {
        Ok(Self {
            backend: PlatformClicker::new()?,
            delay_ms,
        })
    }

    pub fn click(&mut self, button: MouseButton, click_type: ClickType) -> Result<()> {
        self.backend.click(button, click_type)
    }

    pub fn set_delay(&mut self, delay_ms: u64) {
        self.delay_ms = delay_ms;
    }
}
