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

impl ToString for MouseButton {
    fn to_string(&self) -> String {
        match self {
            MouseButton::Left => "Left".to_string(),
            MouseButton::Right => "Right".to_string(),
            MouseButton::Middle => "Middle".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClickType {
    Single,
    Double,
}

impl ToString for ClickType {
    fn to_string(&self) -> String {
        match self {
            ClickType::Single => "Single".to_string(),
            ClickType::Double => "Double".to_string(),
        }
    }
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
