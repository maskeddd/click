use std::{fmt, time::Duration};

use anyhow::Result;

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod linux;
        pub use linux::PlatformInput;
    } else if #[cfg(target_os = "macos")] {
        mod macos;
        pub use macos::PlatformInput;
    } else if #[cfg(target_os = "windows")] {
        mod windows;
        pub use windows::PlatformInput;
    } else {
        compile_error!("Unsupported operating system");
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl fmt::Display for MouseButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MouseButton::Left => write!(f, "Left"),
            MouseButton::Right => write!(f, "Right"),
            MouseButton::Middle => write!(f, "Middle"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClickAction {
    Single,
    Double,
}

impl fmt::Display for ClickAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClickAction::Single => write!(f, "Single"),
            ClickAction::Double => write!(f, "Double"),
        }
    }
}

pub trait InputBackend: Send {
    fn click(&mut self, button: MouseButton) -> Result<()>;
}

pub struct InputHandler {
    backend: PlatformInput,
}

impl InputHandler {
    pub fn new() -> Result<Self> {
        Ok(Self {
            backend: PlatformInput::new()?,
        })
    }

    pub fn click(&mut self, button: MouseButton, click_action: ClickAction) -> Result<()> {
        match click_action {
            ClickAction::Single => {
                self.backend.click(button)?;
            }
            ClickAction::Double => {
                self.backend.click(button)?;
                std::thread::sleep(Duration::from_millis(50));
                self.backend.click(button)?;
            }
        }
        Ok(())
    }
}
