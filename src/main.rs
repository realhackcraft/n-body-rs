use std::f32::consts::PI;

use bevy::{
    input::common_conditions::input_just_pressed, log::tracing_subscriber::fmt::time, prelude::*,
    window::WindowMode,
};

use rand::Rng;

use crate::{
    camera::CameraPlugin,
    cursor::{WorldCursorCoords, WorldCursorPlugin},
};

#[cfg(feature = "debug_inspector")]
use bevy_inspector_egui::InspectorOptions;

mod camera;
mod cursor;
mod trail;
mod window;

/// Defines a physical object
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Object;

/// Position of a body in world space.
/// Split into previous/current for interpolation.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Position {
    pub current: Vec2,
    pub previous: Vec2,
}

impl Position {
    fn new(position: Vec2) -> Position {
        Position {
            current: position,
            previous: position,
        }
    }
}

/// Linear velocity (m/s)
#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

/// Linear acceleration (m/s²)
#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut)]
pub struct Acceleration(pub Vec2);

/// Mass (kg). Must be positive and nonzero.
#[derive(Component, Clone, Copy, Debug)]
pub struct Mass(pub f32);

impl Default for Mass {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Bundle, Clone, Debug, Default)]
pub struct PhysicsBundle {
    pub position: Position,
    pub velocity: Velocity,
    pub acceleration: Acceleration,
    pub mass: Mass,
    pub object: Object,

    /// Bevy transform so renderer knows where to draw the entity.
    pub transform: Transform,
    /// Required for spatial entities (automatically maintained by Bevy).
    pub global_transform: GlobalTransform,
}

#[derive(Resource, States, Clone, Copy, Eq, PartialEq, Debug, Hash, Default)]
enum SimulationState {
    #[default]
    Running,
    Paused,
}

#[derive(Resource)]
#[cfg_attr(
    feature = "debug_inspector",
    derive(Reflect, InspectorOptions),
    reflect(Resource)
)]
pub struct ObjectAssets(pub Handle<Mesh>);

fn main() {
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            title: "N-body Simulator".into(),
            mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
            ..default()
        }),
        ..default()
    };

    App::new()
        .insert_resource(Time::<Fixed>::from_hz(20.0))
        .add_plugins(DefaultPlugins.set(window_plugin))
        .add_plugins(CameraPlugin)
        .add_plugins(WorldCursorPlugin)
        // .add_plugins(TrailPlugin)
        .add_systems(Startup, initialize_assets)
        .add_systems(
            Update,
            (
                spawn_object,
                interpolate_visuals,
                toggle_pause.run_if(input_just_pressed(KeyCode::Space)),
            ),
        )
        .add_systems(
            FixedUpdate,
            physics_step.run_if(in_state(SimulationState::Running)),
        )
        .insert_state(SimulationState::Running)
        .run();
}

fn initialize_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let asset = ObjectAssets({
        let radius = 5.;
        meshes.add(Circle::new(radius))
    });

    commands.insert_resource(asset);
}

fn spawn_object(
    mut commands: Commands,
    object_assets: Res<ObjectAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    cursor: Query<&WorldCursorCoords>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        let mut rng = rand::rng();
        let color = Color::oklch(0.7, 0.159, rng.random_range(0.0..360.));
        let color_material = materials.add(color);

        let mass = 4.;
        let density = 2.;

        let position = cursor.single().unwrap().0;
        commands.spawn((
            PhysicsBundle {
                position: Position::new(position),
                acceleration: Acceleration(Vec2::new(0., 2.)),
                transform: Transform::from_translation(position.extend(0.))
                    .with_scale(Vec3::splat((3. * mass / (density * 4. * PI)).cbrt())),
                ..Default::default()
            },
            Mesh2d(object_assets.0.clone()),
            MeshMaterial2d(color_material),
        ));
    }
}

/// Press SPACE to toggle pause/resume
fn toggle_pause(
    mut next_state: ResMut<NextState<SimulationState>>,
    state: Res<State<SimulationState>>,
    mut time: ResMut<Time<Virtual>>,
) {
    match state.get() {
        SimulationState::Running => {
            next_state.set(SimulationState::Paused);
            time.pause();
        }
        SimulationState::Paused => {
            next_state.set(SimulationState::Running);
            time.unpause();
        }
    }
}

fn physics_step(
    mut q: Query<(&mut Position, &mut Velocity, &Acceleration)>,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta_secs();
    for (mut pos, mut vel, acc) in &mut q {
        vel.0 += acc.0 * dt;
        pos.previous = pos.current;
        pos.current += vel.0 * dt;
    }
}

fn interpolate_visuals(
    mut q: Query<(&Position, &mut Transform), With<Object>>,
    time: Res<Time>,
    fixed_time: Res<Time<Fixed>>,
) {
    let fixed_dt = fixed_time.delta_secs();
    let alpha = ((time.elapsed_secs() - fixed_time.elapsed_secs()) / fixed_dt).clamp(0.0, 1.0);

    for (pos, mut transform) in q.iter_mut() {
        transform.translation = pos.previous.lerp(pos.current, alpha).extend(0.);
    }
}
