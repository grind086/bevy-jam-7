//! Player-specific behavior.

use avian2d::prelude::{Collider, LockedAxes, Mass, RigidBody, Sensor};
use bevy::{ecs::relationship::RelatedSpawner, prelude::*};
use rand::seq::IndexedRandom;

use crate::{
    AppSystems, PausableSystems,
    animation::{Animation, AnimationPlayer, AnimationPlayerState},
    asset_tracking::LoadResource,
    audio::sound_effect,
    demo::movement::{FootSensorOf, MovementController, MovementIntent, OnGround},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<PlayerAssets>();

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (
            record_player_directional_input.in_set(AppSystems::RecordInput),
            (update_animation_movement, trigger_step_sound_effect),
        )
            .chain()
            .run_if(in_state(Screen::Gameplay))
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
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 12, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    (
        Name::new("Player"),
        Player,
        Sprite {
            image: player_assets.ducky.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
            }),
            custom_size: Some(Vec2::splat(2.)),
            ..default()
        },
        AnimationPlayer::from(player_assets.idle_anim.clone()),
        Mass(1.5),
        Children::spawn(SpawnWith(|related: &mut RelatedSpawner<ChildOf>| {
            related.spawn((
                Transform::from_translation(0.5 * Vec3::NEG_Y),
                // Collider::capsule(0.40, 0.2),
                Collider::rectangle(0.8, 1.0),
            ));
            related.spawn((
                Sensor,
                FootSensorOf(related.target_entity()),
                Transform::from_translation(1.0 * Vec3::NEG_Y),
                Collider::rectangle(0.70, 0.1),
            ));
        })),
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED,
        Transform::from_translation(position.extend(0.0)),
        MovementController {
            max_speed,
            ..default()
        },
        // player_animation,
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
    mut intent: Single<&mut MovementIntent, With<Player>>,
) {
    // Collect directional input.
    let lt = input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let rt = input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    intent.direction = (rt as i8 - lt as i8).into();
    intent.jump = input.just_pressed(KeyCode::Space);
}

fn update_animation_movement(
    assets: Res<PlayerAssets>,
    mut player_query: Query<(
        &MovementIntent,
        Option<&OnGround>,
        &mut Sprite,
        &mut AnimationPlayer,
    )>,
) {
    for (intent, on_ground, mut sprite, mut animation) in &mut player_query {
        if intent.direction != 0.0 {
            sprite.flip_x = intent.direction < 0.0;
        }

        let next_anim = if on_ground.is_none_or(|g| **g) {
            if intent.direction == 0.0 {
                &assets.idle_anim
            } else {
                &assets.walk_anim
            }
        } else {
            &assets.fall_anim
        };

        if next_anim.id() != animation.animation.id() {
            animation.animation = next_anim.clone();
        }
    }
}

/// If the player is moving, play a step sound effect synchronized with the
/// animation.
fn trigger_step_sound_effect(
    mut commands: Commands,
    player_assets: If<Res<PlayerAssets>>,
    mut step_query: Query<(&AnimationPlayer, &AnimationPlayerState), Changed<AnimationPlayerState>>,
) {
    for (player, state) in &mut step_query {
        if player.animation.id() == player_assets.walk_anim.id()
            && (state.frame_index() == 2 || state.frame_index() == 5)
        {
            let rng = &mut rand::rng();
            let random_step = player_assets.steps.choose(rng).unwrap().clone();
            commands.spawn(sound_effect(random_step));
        }
    }
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
    pub idle_anim: Handle<Animation>,
    pub walk_anim: Handle<Animation>,
    pub fall_anim: Handle<Animation>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let mut animations = world.resource_mut::<Assets<Animation>>();
        let idle_anim = animations.add(Animation::from_frame_range_and_millis(0..6, 500));
        let walk_anim = animations.add(Animation::from_frame_range_and_millis(6..12, 50));
        let fall_anim = animations.add(Animation::from_frame_range_and_millis(42..48, 300));

        let assets = world.resource::<AssetServer>();
        Self {
            ducky: assets.load("images/Hero_001.png"),
            steps: vec![
                assets.load("audio/sound_effects/step1.ogg"),
                assets.load("audio/sound_effects/step2.ogg"),
                assets.load("audio/sound_effects/step3.ogg"),
                assets.load("audio/sound_effects/step4.ogg"),
            ],
            idle_anim,
            walk_anim,
            fall_anim,
        }
    }
}
