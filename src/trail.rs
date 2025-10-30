use bevy::prelude::*;
use bevy_hanabi::prelude::*;

#[cfg(feature = "debug_inspector")]
use bevy_inspector_egui::InspectorOptions;

/// The plugin for managing the trail of anything that's visible.
pub struct TrailPlugin;

#[derive(Resource)]
#[cfg_attr(
    feature = "debug_inspector",
    derive(Reflect, InspectorOptions),
    reflect(Resource)
)]
pub struct TrailEffectHandle(pub Handle<EffectAsset>);

impl Plugin for TrailPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_trail);
    }
}

pub fn setup_trail(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    // Continueous stream, spawns 1 per frame
    let spawner = SpawnerSettings::rate(60.0.into());
    let writer = ExprWriter::new();

    let age = writer.lit(0.).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);
    let lifetime = writer.add_property("lifetime", 0.25.into());
    let init_lifetime =
        SetAttributeModifier::new(Attribute::LIFETIME, writer.prop(lifetime).expr());

    let z = writer.add_property("z", 0.0.into());
    let pos = writer.lit(0.).vec3(writer.lit(0.), writer.prop(z)).expr();
    let init_pos = SetAttributeModifier::new(Attribute::POSITION, pos);

    let roundness = writer.lit(1.0).expr();

    let spawn_color = writer.add_property("spawn_color", Vec4::ONE.into());
    let color_updater = SetAttributeModifier::new(
        Attribute::COLOR,
        writer
            .prop(spawn_color)
            .x()
            .vec3(writer.prop(spawn_color).y(), writer.prop(spawn_color).z())
            .vec4_xyz_w(
                writer.prop(spawn_color).w()
                    * (writer.lit(1.)
                        - (writer.attr(Attribute::AGE) / writer.attr(Attribute::LIFETIME))),
            )
            .pack4x8unorm()
            .expr(),
    );

    // Interpolate the diameter from the provided diameter to half of it
    let diameter = writer.add_property("diameter", 26.0.into());
    let diameter_updater = SetAttributeModifier::new(
        Attribute::SIZE,
        (writer.prop(diameter)
            * (writer.lit(1.) - (writer.attr(Attribute::AGE) / writer.attr(Attribute::LIFETIME)))
            + writer.prop(diameter) / writer.lit(2.)
                * (writer.attr(Attribute::AGE) / writer.attr(Attribute::LIFETIME)))
        .expr(),
    );

    commands.insert_resource(TrailEffectHandle(
        effects.add(
            // 60 * 1 (max duration) = 60, allocate 70 for wiggle room
            // NOTE: Change if need to increase max lifetime
            EffectAsset::new(70, spawner, writer.finish())
                .with_name("trail")
                .with_motion_integration(MotionIntegration::None)
                .init(init_pos)
                .init(init_age)
                .init(init_lifetime)
                .render(RoundModifier { roundness })
                .update(diameter_updater)
                .update(color_updater),
        ),
    ));
}
