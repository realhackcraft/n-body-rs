use bevy::{prelude::*, window::PrimaryWindow};
#[cfg(feature = "debug_inspector")]
use bevy_inspector_egui::InspectorOptions;

use crate::camera::GameCamera;

/// A plugin to update the world cursor.
pub struct WorldCursorPlugin;

impl Plugin for WorldCursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, cursor_world_coord);

        #[cfg(feature = "debug_inspector")]
        app.register_type::<WorldCursorCoords>();
    }
}

/// Stores the cursor position in terms of the world.
#[derive(Component, Default)]
#[cfg_attr(feature = "debug_inspector", derive(Reflect, InspectorOptions))]
pub struct WorldCursorCoords(pub Vec2);

#[allow(clippy::needless_pass_by_value)]
fn cursor_world_coord(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut camera: Query<(&Camera, &GlobalTransform, &mut WorldCursorCoords), With<GameCamera>>,
) {
    let (camera, camera_transform, mut worldcursor) = camera.single_mut().unwrap();
    let window = primary_window.single().unwrap();

    // Check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window.cursor_position().and_then(|cursor| {
        let world_cord = camera.viewport_to_world_2d(camera_transform, cursor);
        world_cord.ok()
    }) {
        worldcursor.0 = world_position;
    }
}
