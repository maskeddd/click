use std::time::Duration;

use super::{ClickType, ClickerBackend, MouseButton};
use anyhow::Result;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_MOUSE, MOUSE_EVENT_FLAGS, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
    MOUSEINPUT, SendInput,
};

pub struct PlatformClicker;

impl PlatformClicker {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    fn emit_click(&mut self, button: MouseButton) {
        let (flag_down, flag_up) = self.button_to_flags(button);

        let input_down = self.create_mouse_input(flag_down);
        let input_up = self.create_mouse_input(flag_up);

        unsafe {
            SendInput(&[input_down, input_up], std::mem::size_of::<INPUT>() as i32);
        }
    }

    fn create_mouse_input(&self, flag: MOUSE_EVENT_FLAGS) -> INPUT {
        INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: flag,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    fn button_to_flags(&self, button: MouseButton) -> (MOUSE_EVENT_FLAGS, MOUSE_EVENT_FLAGS) {
        match button {
            MouseButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
            MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
            MouseButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
        }
    }
}

impl ClickerBackend for PlatformClicker {
    fn click(&mut self, button: MouseButton, click_type: ClickType) -> Result<()> {
        match click_type {
            ClickType::Single => {
                self.emit_click(button);
            }
            ClickType::Double => {
                self.emit_click(button);
                std::thread::sleep(Duration::from_millis(40));
                self.emit_click(button);
            }
        }
        Ok(())
    }
}
