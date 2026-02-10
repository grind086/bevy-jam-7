//! Player-specific behavior.

use avian2d::prelude::{Collider, LockedAxes, RigidBody};
use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    demo::{animation::PlayerAnimation, movement::MovementController},
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<PlayerAssets>();

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        record_player_directional_input
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems),
    );

    // Update camera position
    app.add_systems(
        PostUpdate,
        update_player_camera_position.before(TransformSystems::Propagate),
    );
}

/// The player character.
pub fn player(
    position: Vec2,
    max_speed: f32,
    player_assets: &PlayerAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = PlayerAnimation::new();

    (
        Name::new("Player"),
        Player,
        Sprite {
            image: player_assets.ducky.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: player_animation.get_atlas_index(),
            }),
            custom_size: Some(Vec2::new(1.0, 1.8)),
            ..default()
        },
        Collider::capsule(0.45, 0.8),
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED,
        Transform::from_translation(position.extend(0.0)),
        MovementController {
            max_speed,
            ..default()
        },
        player_animation,
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct PlayerCamera;

fn record_player_directional_input(
    input: Res<ButtonInput<KeyCode>>,
    mut controller: Single<&mut MovementController, With<Player>>,
) {
    // Collect directional input.
    let mut intent = Vec2::ZERO;
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        intent.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        intent.x += 1.0;
    }

    // Normalize intent so that diagonal movement is the same speed as horizontal / vertical.
    // This should be omitted if the input comes from an analog stick instead.
    let intent = intent.normalize_or_zero();

    controller.intent = intent;
    controller.jump = input.pressed(KeyCode::Space);
}

fn update_player_camera_position(
    player: Single<&GlobalTransform, (With<Player>, Without<PlayerCamera>)>,
    mut camera: Single<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    camera.translation = player.translation();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    ducky: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            ducky: assets.load("images/ducky.png"),
            steps: vec![
                assets.load("audio/sound_effects/step1.ogg"),
                assets.load("audio/sound_effects/step2.ogg"),
                assets.load("audio/sound_effects/step3.ogg"),
                assets.load("audio/sound_effects/step4.ogg"),
            ],
        }
    }
}
