use crate::input::Coordinates;

use super::{InputBackend, MouseButton};
use anyhow::Result;
use evdev::{
    AttributeSet, EventType, InputEvent, KeyCode, RelativeAxisCode, uinput::VirtualDevice,
};

pub struct PlatformInput {
    device: VirtualDevice,
}

impl PlatformInput {
    pub fn new() -> Result<Self> {
        let keys = AttributeSet::<KeyCode>::from_iter([
            KeyCode::BTN_LEFT,
            KeyCode::BTN_RIGHT,
            KeyCode::BTN_MIDDLE,
        ]);

        let axes = AttributeSet::<RelativeAxisCode>::from_iter([
            RelativeAxisCode::REL_X,
            RelativeAxisCode::REL_Y,
        ]);

        let device = VirtualDevice::builder()?
            .name("click-virtual-device")
            .with_keys(&keys)?
            .with_relative_axes(&axes)?
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

impl InputBackend for PlatformInput {
    fn click(&mut self, button: MouseButton) -> Result<()> {
        let key = self.button_to_key(button);

        self.device
            .emit(&[InputEvent::new(EventType::KEY.0, key.code(), 1)])?;
        self.device
            .emit(&[InputEvent::new(EventType::KEY.0, key.code(), 0)])?;

        Ok(())
    }

    fn move_to(&mut self, coords: Coordinates) -> Result<()> {
        self.device.emit(&[
            InputEvent::new(EventType::RELATIVE.0, RelativeAxisCode::REL_X.0, -32767),
            InputEvent::new(EventType::RELATIVE.0, RelativeAxisCode::REL_Y.0, -32767),
        ])?;

        self.device.emit(&[
            InputEvent::new(EventType::RELATIVE.0, RelativeAxisCode::REL_X.0, coords.x),
            InputEvent::new(EventType::RELATIVE.0, RelativeAxisCode::REL_Y.0, coords.y),
        ])?;

        Ok(())
    }
}
