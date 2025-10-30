use bevy::{prelude::*, window::WindowResized};

use crate::{cursor::WorldCursorCoords, window::has_resize_event};

/// A plugin for managing the main (and only) camera.
pub struct CameraPlugin;

/// A marker component to identify the main game camera.
#[derive(Component)]
pub struct GameCamera;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::hsla(0., 0., 0.05, 1.)))
            .add_systems(Startup, setup);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        GameCamera,
        WorldCursorCoords::default(),
        Name::new("Game Camera"),
    ));
}

/// From a world position, the current window, and the camera's Transform component, compute
/// whether the position is inside of the window or not.
pub fn in_bound(
    position: Vec3,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    tolerance: f32,
) -> bool {
    // Adjust for camera transforms
    let position = position / camera_transform.scale() - camera_transform.translation();

    if let (Some(ndc_pos), Some(ndc_tolerance)) = (
        camera.world_to_ndc(camera_transform, position),
        camera.world_to_ndc(camera_transform, Vec3::X * tolerance + Vec3::Y * tolerance),
    ) {
        // Adjust NDC bounds by the calculated tolerance in NDC space.
        let left_bound = -1.0 - ndc_tolerance.x;
        let right_bound = 1.0 + ndc_tolerance.x;
        let bottom_bound = -1.0 - ndc_tolerance.y;
        let top_bound = 1.0 + ndc_tolerance.y;

        // Check NDC position against NDC bounds with added tolerance
        ndc_pos.x > left_bound
            && ndc_pos.x < right_bound
            && ndc_pos.y > bottom_bound
            && ndc_pos.y < top_bound
    } else {
        false // If failed, the point is out of bounds
    }
}
