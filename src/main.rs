use std::f32::consts::PI;

use bevy::{prelude::*, window::WindowMode};

use rand::Rng;

use crate::{
    camera::CameraPlugin,
    cursor::{WorldCursorCoords, WorldCursorPlugin},
    trail::TrailPlugin,
};

#[cfg(feature = "debug_inspector")]
use bevy_inspector_egui::InspectorOptions;

mod camera;
mod cursor;
mod trail;
mod window;

#[derive(Component)]
struct Mass(f32);

#[derive(Resource)]
struct Running(bool);

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
            title: "Blob Shooter".into(),
            mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
            ..default()
        }),
        ..default()
    };

    App::new()
        .add_plugins(DefaultPlugins.set(window_plugin))
        .add_plugins(CameraPlugin)
        .add_plugins(WorldCursorPlugin)
        // .add_plugins(TrailPlugin)
        .add_systems(Startup, initialize_assets)
        .add_systems(Update, (spawn_object, pause_sim))
        .add_systems(FixedUpdate, simulate)
        .insert_resource(Running(true))
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

        commands.spawn((
            Mass(mass),
            Transform::from_translation(cursor.single().unwrap().0.extend(0.))
                .with_scale(Vec3::splat((3. * mass * density / (4. * PI)).cbrt())),
            Mesh2d(object_assets.0.clone()),
            MeshMaterial2d(color_material),
        ));
    }
}

fn pause_sim(mut running: ResMut<Running>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        running.0 = !running.0;
    }
}

fn simulate() {}
