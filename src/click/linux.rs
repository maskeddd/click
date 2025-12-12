use super::{ClickType, ClickerBackend, MouseButton};
use anyhow::Result;
use evdev::{AttributeSet, EventType, InputEvent, KeyCode, uinput::VirtualDevice};

pub struct PlatformClicker {
    device: VirtualDevice,
}

impl PlatformClicker {
    pub fn new() -> Result<Self> {
        let keys = AttributeSet::<KeyCode>::from_iter([
            KeyCode::BTN_LEFT,
            KeyCode::BTN_RIGHT,
            KeyCode::BTN_MIDDLE,
        ]);

        let device = VirtualDevice::builder()?
            .name("clicker-virtual-device")
            .with_keys(&keys)?
            .build()?;

        Ok(Self { device })
    }

    fn button_to_key(&self, button: MouseButton) -> KeyCode {
        match button {
            MouseButton::Left => KeyCode::BTN_LEFT,
            MouseButton::Right => KeyCode::BTN_RIGHT,
            MouseButton::Middle => KeyCode::BTN_MIDDLE,
        }
    }
}

impl ClickerBackend for PlatformClicker {
    fn click(&mut self, button: MouseButton, click_type: ClickType) -> Result<()> {
        let key = self.button_to_key(button);

        match click_type {
            ClickType::Single => {
                self.device
                    .emit(&[InputEvent::new(EventType::KEY.0, key.code(), 1)])?;
                self.device
                    .emit(&[InputEvent::new(EventType::KEY.0, key.code(), 0)])?;
            }
            ClickType::Hold => {
                self.device
                    .emit(&[InputEvent::new(EventType::KEY.0, key.code(), 1)])?;
            }
        }

        Ok(())
    }
}
