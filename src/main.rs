use std::f32::consts::PI;

use bevy_inspector_egui::prelude::*;

use bevy::{input::common_conditions::input_just_pressed, prelude::*, window::WindowMode};

use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
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
#[derive(Component, Clone, Copy, Debug, Default, Reflect, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct Object;

/// Position of a body in world space.
#[derive(Component, Clone, Copy, Debug, Default, Reflect, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct Position(Vec2);

#[derive(Component, Clone, Copy, Debug, Default, Reflect, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct LastPos(Vec2);

/// Linear velocity (m/s)
#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut, Reflect, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct Velocity(pub Vec2);

/// Linear force (N)
#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut, Reflect, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct Force(pub Vec2);

/// Mass (kg). Must be positive and nonzero.
#[derive(Component, Clone, Copy, Debug, Reflect, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct Mass(pub f32);

impl Default for Mass {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Bundle, Clone, Debug, Default)]
pub struct PhysicsBundle {
    pub position: Position,
    pub last_pos: LastPos,
    pub velocity: Velocity,
    pub force: Force,
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

const GRAVITY_CONSTANT: f32 = 6.67e-11;

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
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
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
            (
                interact_bodies.run_if(in_state(SimulationState::Running)),
                physics_step.run_if(in_state(SimulationState::Running)),
            )
                .chain(),
        )
        .insert_state(SimulationState::Running)
        .register_type::<Position>()
        .register_type::<LastPos>()
        .register_type::<Velocity>()
        .register_type::<Force>()
        .register_type::<Mass>()
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
                position: Position(position),
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

fn interact_bodies(mut query: Query<(&Mass, &Position, &mut Force)>) {
    let mut iter = query.iter_combinations_mut();
    while let Some([(Mass(m1), pos1, mut f1), (Mass(m2), pos2, mut f2)]) = iter.fetch_next() {
        let delta = pos2.0 - pos1.0;
        let distance_sq: f32 = delta.length_squared();

        // G / r^2
        let f = GRAVITY_CONSTANT / distance_sq;
        let force = f * *m1 * *m2 * delta.normalize();
        f1.0 += force;
        f2.0 -= force;
    }
}

fn physics_step(
    mut q: Query<(
        &mut Position,
        &mut LastPos,
        &mut Velocity,
        &mut Force,
        &Mass,
    )>,
    time: Res<Time<Fixed>>,
) {
    // convert real seconds into simulated years
    let dt = time.delta_secs() * 3600. * 24. * 356.25;

    for (mut pos, mut last_pos, mut vel, mut f, m) in &mut q {
        let acc = f.0 / m.0;
        vel.0 += acc * dt;
        last_pos.0 = pos.0;
        pos.0 += vel.0 * dt;
        f.0 = Vec2::splat(0.);
    }
}

fn interpolate_visuals(
    mut q: Query<(&Position, &LastPos, &mut Transform), With<Object>>,
    time: Res<Time>,
    fixed_time: Res<Time<Fixed>>,
) {
    let fixed_dt = fixed_time.delta_secs();
    let alpha = ((time.elapsed_secs() - fixed_time.elapsed_secs()) / fixed_dt).clamp(0.0, 1.0);

    for (pos, last_pos, mut transform) in q.iter_mut() {
        transform.translation = last_pos.0.lerp(pos.0, alpha).extend(0.);
    }
}
