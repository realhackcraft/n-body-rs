use bevy::{prelude::*, window::WindowResized};

/// A run condition that is true of the window has been resized.
#[allow(clippy::needless_pass_by_value)]
pub fn has_resize_event(resize_events: MessageReader<WindowResized>) -> bool {
    !resize_events.is_empty()
}
