mod accessibility;

use super::{InputBackend, MouseButton};
use anyhow::Result;
use objc2::rc::autoreleasepool;
use objc2_app_kit::NSEvent;
use objc2_core_foundation::{CFRetained, CGPoint};
use objc2_core_graphics::{
    CGDisplayPixelsHigh, CGEvent, CGEventSource, CGEventSourceStateID, CGEventTapLocation,
    CGEventType, CGMainDisplayID, CGMouseButton,
};

pub struct PlatformInput {
    source: Option<CFRetained<CGEventSource>>,
    display_height: f64,
}

unsafe impl Send for PlatformInput {}

impl PlatformInput {
    pub fn new() -> Result<Self> {
        // We need accessibility permissions
        accessibility::ensure_trust();

        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState);

        Ok(Self {
            source,
            display_height: CGDisplayPixelsHigh(CGMainDisplayID()) as f64,
        })
    }

    fn mouse_button_to_cg(button: MouseButton) -> CGMouseButton {
        match button {
            MouseButton::Left => CGMouseButton::Left,
            MouseButton::Right => CGMouseButton::Right,
            MouseButton::Middle => CGMouseButton::Center,
        }
    }

    fn get_event_types(button: MouseButton) -> (CGEventType, CGEventType) {
        match button {
            MouseButton::Left => (CGEventType::LeftMouseDown, CGEventType::LeftMouseUp),
            MouseButton::Right => (CGEventType::RightMouseDown, CGEventType::RightMouseUp),
            MouseButton::Middle => (CGEventType::OtherMouseDown, CGEventType::OtherMouseUp),
        }
    }

    fn get_mouse_position(&self) -> CGPoint {
        let pos = NSEvent::mouseLocation();

        CGPoint {
            x: pos.x,
            y: self.display_height - pos.y,
        }
    }
}

impl InputBackend for PlatformInput {
    fn click(&mut self, button: MouseButton) -> Result<()> {
        let pos = self.get_mouse_position();
        let cg_button = Self::mouse_button_to_cg(button);
        let (down_type, up_type) = Self::get_event_types(button);
        let source = self.source.as_deref();

        // autoreleasepool to prevent leaking
        autoreleasepool(|_| {
            for event_type in [down_type, up_type] {
                if let Some(event) = CGEvent::new_mouse_event(source, event_type, pos, cg_button) {
                    CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&*event));
                }
            }
        });

        Ok(())
    }
}
