//! Player-specific behavior.

use avian2d::prelude::{Collider, CollisionLayers, LinearVelocity};
use bevy::{prelude::*, ui_widgets::observe};
use rand::seq::IndexedRandom;

use crate::{
    AppSystems, PausableSystems,
    animation::{Animation, AnimationEvent, AnimationPlayer},
    asset_tracking::LoadResource,
    audio::sound_effect,
    controller::{CharacterController, CharacterIntent, GroundNormal, character_controller},
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
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 1, 23, Some(UVec2::ONE), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let collider_offset = 0.55 * Vec2::NEG_Y;

    (
        Name::new("Player"),
        Player,
        Transform::from_translation(position.extend(0.0)),
        Visibility::default(),
        character_controller(
            CharacterController {
                max_speed: 20.,
                accel_air: 3.5,
                accel_ground: 35.0,
                decel_ground: 20.0,
                damping_air: 0.3,
                damping_ground: 0.9,
                jump_impulse: 65.0,
                jump_min_ticks: 4,
                jump_max_ticks: 8,
                max_slope_angle: f32::to_radians(60.0),
            },
            Collider::capsule(0.2, 0.45),
            CollisionLayers::player(),
        ),
        children![(
            Sprite {
                image: player_assets.ducky.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: texture_atlas_layout,
                    index: 0,
                }),
                custom_size: Some(Vec2::splat(2.)),
                ..default()
            },
            Transform::from_translation((-collider_offset).extend(0.0)),
            AnimationPlayer::from(player_assets.idle_anim.clone()),
            observe(trigger_step_sound_effect),
        )],
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
    mut intent: Single<&mut CharacterIntent, With<Player>>,
) {
    // Collect directional input.
    let lt = input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let rt = input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);
    let run = !input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    intent.movement = f32::from(rt as i8 - lt as i8) * if run { 1.0 } else { 0.25 };
    intent.jump = input.pressed(KeyCode::Space);
}

fn update_animation_movement(
    assets: Res<PlayerAssets>,
    player: Single<
        (
            &CharacterIntent,
            Option<&GroundNormal>,
            Option<&LinearVelocity>,
            &Children,
        ),
        With<Player>,
    >,
    mut sprites: Query<(&mut Sprite, &mut AnimationPlayer)>,
) {
    let (intent, ground_norm, velocity, children) = player.into_inner();
    let Ok((mut sprite, mut animation)) = sprites.get_mut(children[0]) else {
        return;
    };

    if intent.movement != 0.0 {
        sprite.flip_x = intent.movement < 0.0;
    }

    let next_anim = if ground_norm.is_none_or(GroundNormal::is_grounded) {
        let vx = velocity.map_or(0.0, |v| v.x.abs());
        if vx < 0.1 {
            &assets.idle_anim
        } else if vx < 10.0 {
            &assets.walk_anim
        } else {
            &assets.run_anim
        }
    } else {
        let vy = velocity.map_or(-1.0, |v| v.y);
        if vy.abs() < 0.5 {
            &assets.peak_anim
        } else if vy > 0.0 {
            &assets.jump_anim
        } else {
            &assets.fall_anim
        }
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
        commands.spawn(sound_effect(random_step, 0.3));
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
    pub run_anim: Handle<Animation>,
    pub jump_anim: Handle<Animation>,
    pub peak_anim: Handle<Animation>,
    pub fall_anim: Handle<Animation>,
}

impl PlayerAssets {
    pub const STEP_MARKER: usize = 0;
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let mut animations = world.resource_mut::<Assets<Animation>>();
        let idle_anim = animations.add(Animation::from_frame_range_and_millis(0..4, 250));
        let walk_anim = animations.add(
            Animation::from_frame_range_and_millis(4..12, 50)
                .with_marker(Self::STEP_MARKER, [2, 6]),
        );
        let run_anim = animations.add(
            Animation::from_frame_range_and_millis(12..20, 50)
                .with_marker(Self::STEP_MARKER, [3, 7]),
        );
        let jump_anim = animations.add(Animation::from_frame_range_and_millis(20..21, 50));
        let peak_anim = animations.add(Animation::from_frame_range_and_millis(21..22, 50));
        let fall_anim = animations.add(Animation::from_frame_range_and_millis(22..23, 50));

        let assets = world.resource::<AssetServer>();
        Self {
            ducky: assets.load("images/player.png"),
            steps: vec![
                assets.load("audio/sound_effects/steps/grass1.ogg"),
                assets.load("audio/sound_effects/steps/grass2.ogg"),
                assets.load("audio/sound_effects/steps/grass3.ogg"),
                assets.load("audio/sound_effects/steps/grass4.ogg"),
            ],
            idle_anim,
            walk_anim,
            run_anim,
            jump_anim,
            peak_anim,
            fall_anim,
        }
    }
}
