use super::{InputBackend, MouseButton};
use anyhow::Result;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_MOUSE, MOUSE_EVENT_FLAGS, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
    MOUSEINPUT, SendInput,
};

pub struct PlatformInput;

impl PlatformInput {
    pub fn new() -> Result<Self> {
        // lower timer resolution
        unsafe {
            windows::Win32::Media::timeBeginPeriod(1);
        }

        Ok(Self)
    }

    fn create_mouse_input(flag: MOUSE_EVENT_FLAGS) -> INPUT {
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

    fn button_to_flags(button: MouseButton) -> (MOUSE_EVENT_FLAGS, MOUSE_EVENT_FLAGS) {
        match button {
            MouseButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
            MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
            MouseButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
        }
    }
}

impl InputBackend for PlatformInput {
    fn click(&mut self, button: MouseButton) -> Result<()> {
        let (flag_down, flag_up) = Self::button_to_flags(button);

        let inputs = [
            Self::create_mouse_input(flag_down),
            Self::create_mouse_input(flag_up),
        ];

        unsafe {
            SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        }

        Ok(())
    }
}
