mod accessibility;

use super::{ClickType, ClickerBackend, MouseButton};
use anyhow::Result;
use objc2_app_kit::NSEvent;
use objc2_core_foundation::{CFRetained, CGPoint};
use objc2_core_graphics::{
    CGDisplayPixelsHigh, CGEvent, CGEventSource, CGEventSourceStateID, CGEventTapLocation,
    CGEventType, CGMainDisplayID, CGMouseButton,
};

pub struct PlatformClicker {
    source: Option<CFRetained<CGEventSource>>,
    display_height: f64,
}

unsafe impl Send for PlatformClicker {}

impl PlatformClicker {
    pub fn new() -> Result<Self> {
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
}

impl ClickerBackend for PlatformClicker {
    fn click(&mut self, button: MouseButton, click_type: ClickType) -> Result<()> {
        let pos = NSEvent::mouseLocation();
        let cg_button = Self::mouse_button_to_cg(button);
        let (down_type, up_type) = Self::get_event_types(button);

        let flipped = CGPoint {
            x: pos.x,
            y: self.display_height - pos.y,
        };

        let source_ref = self.source.as_ref().map(|r| &**r);

        // Use correct event types for the button
        if let Some(down) = CGEvent::new_mouse_event(source_ref, down_type, flipped, cg_button) {
            CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&*down));
        }

        if let Some(up) = CGEvent::new_mouse_event(source_ref, up_type, flipped, cg_button) {
            CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&*up));
        }

        Ok(())
    }
}
