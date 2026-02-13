//! Player-specific behavior.

use avian2d::prelude::{Collider, CollisionLayers};
use bevy::{prelude::*, ui_widgets::observe};
use rand::seq::IndexedRandom;

use crate::{
    AppSystems, PausableSystems,
    animation::{Animation, AnimationEvent, AnimationPlayer},
    asset_tracking::LoadResource,
    audio::sound_effect,
    demo::movement::{GroundNormal, MovementController, MovementIntent, movement_controller},
    physics::GamePhysicsLayersExt,
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<PlayerAssets>();

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (
            record_player_directional_input.in_set(AppSystems::RecordInput),
            update_animation_movement,
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
    player_assets: &PlayerAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 12, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let collider_offset = 0.5 * Vec2::NEG_Y;

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
        Transform::from_translation((position - collider_offset).extend(0.0)),
        movement_controller(
            MovementController {
                max_speed: 20.,
                accel_ground: 1.5,
                accel_air: 0.1,
                jump_strength: 20.,
                damping_factor_air: 0.3,
                damping_factor_ground: 2.5,
                max_slope_angle: f32::to_radians(60.0),
                ..default()
            },
            Collider::capsule(0.35, 0.2),
            // Collider::rectangle(0.8, 1.0),
            collider_offset,
            CollisionLayers::player(),
        ),
        observe(trigger_step_sound_effect),
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
    intent.jump = input.pressed(KeyCode::Space);
}

fn update_animation_movement(
    assets: Res<PlayerAssets>,
    player: Single<
        (
            &MovementIntent,
            Option<&GroundNormal>,
            &mut Sprite,
            &mut AnimationPlayer,
        ),
        With<Player>,
    >,
) {
    let (intent, ground_norm, mut sprite, mut animation) = player.into_inner();

    if intent.direction != 0.0 {
        sprite.flip_x = intent.direction < 0.0;
    }

    let next_anim = if ground_norm.is_none_or(GroundNormal::is_grounded) {
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

fn trigger_step_sound_effect(
    ev: On<AnimationEvent>,
    player_assets: If<Res<PlayerAssets>>,
    mut commands: Commands,
) {
    if ev.marker == PlayerAssets::STEP_MARKER {
        let rng = &mut rand::rng();
        let random_step = player_assets.steps.choose(rng).unwrap().clone();
        commands.spawn(sound_effect(random_step));
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

impl PlayerAssets {
    pub const STEP_MARKER: usize = 0;
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let mut animations = world.resource_mut::<Assets<Animation>>();
        let idle_anim = animations.add(Animation::from_frame_range_and_millis(0..6, 500));
        let walk_anim = animations.add(
            Animation::from_frame_range_and_millis(6..12, 50)
                .with_marker(Self::STEP_MARKER, [2, 5]),
        );
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
