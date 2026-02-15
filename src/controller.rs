use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{PausableSystems, physics::GamePhysicsLayers};

const CASTER_SHAPE_SCALE: f32 = 0.99;
const CASTER_MAX_DISTANCE: f32 = 0.1;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(PreUpdate, reset_jump_state)
        .add_systems(
            FixedUpdate,
            (
                update_grounded,
                apply_gravity,
                apply_movement_damping,
                apply_intents,
            )
                .chain()
                .in_set(PausableSystems),
        )
        .add_systems(
            PhysicsSchedule,
            (handle_collisions, apply_move_and_slide)
                .chain()
                .in_set(NarrowPhaseSystems::Last),
        );
}

pub fn character_controller(
    settings: CharacterController,
    collider: Collider,
    collision_layers: CollisionLayers,
) -> impl Bundle {
    let mut caster_shape = collider.clone();
    caster_shape.set_scale(Vec2::splat(CASTER_SHAPE_SCALE), 10);

    (
        settings,
        RigidBody::Kinematic,
        LockedAxes::ROTATION_LOCKED,
        CustomPositionIntegration,
        collider,
        collision_layers,
        ShapeCaster::new(caster_shape, Vec2::ZERO, 0.0, Dir2::NEG_Y)
            .with_max_distance(CASTER_MAX_DISTANCE)
            // Removing this allows walking/jumping on top of enemies. Good? Bad?
            .with_query_filter(SpatialQueryFilter::from_mask(
                GamePhysicsLayers::LevelGeometry,
            )),
    )
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(CharacterIntent, GroundNormal, JumpState, MoveAndSlideResult)]
pub struct CharacterController {
    /// Acceleration applied while in the air.
    pub accel_air: f32,

    /// Acceleration applied while on the ground.
    pub accel_ground: f32,

    /// Deceleration applied while on the ground with a neutral [`movement`] intent.
    ///
    /// Generally this should be less than [`accel_ground`], but it isn't required.
    ///
    /// [`movement`]: CharacterIntent::movement
    /// [`accel_ground`]: Self::accel_ground
    pub decel_ground: f32,

    /// Exponential velocity falloff (per second) while in the air.
    pub damping_air: f32,

    /// Exponential velocity falloff (per second) while grounded.
    pub damping_ground: f32,

    /// The impulse to apply when jumping.
    ///
    /// When a character jumps this impulse will be applied for between [`jump_min_ticks`] and
    /// [`jump_max_ticks`] physics timesteps, depending on how long the character's [`jump`] intent
    /// remains `true`.
    ///
    /// [`jump_min_ticks`]: Self::jump_min_ticks
    /// [`jump_max_ticks`]: Self::jump_max_ticks
    /// [`jump`]: CharacterIntent::jump
    pub jump_impulse: f32,

    /// Jump impulses will always be applied for at least this many physics timesteps.
    ///
    /// Increasing this can result in a more consistent minimum jump height, but may cause the
    /// controller to feel less responsive.
    pub jump_min_ticks: u32,

    /// Jump impulses will be applied for at most this many physics timesteps.
    ///
    /// Low values will result in a smaller range of possible jump heights. Large values combined
    /// with a low [`jump_impulse`] can be used for a jetpack type effect.
    ///
    /// If this is less than or equal to [`jump_min_ticks`], jump impulses will always be applied
    /// for exactly [`jump_min_ticks`] physics timesteps.
    ///
    /// [`jump_impulse`]: Self::jump_impulse
    /// [`jump_min_ticks`]: Self::jump_min_ticks
    pub jump_max_ticks: u32,

    /// The maximum angle on which a character can stand and be considered grounded.
    pub max_slope_angle: f32,

    /// The maximum speed that the character can accelerate itself to while on the ground.
    pub max_speed: f32,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CharacterIntent {
    pub movement: f32,
    pub jump: bool,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct GroundNormal(Option<Vec2>);

impl GroundNormal {
    pub fn is_grounded(&self) -> bool {
        self.0.is_some()
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct JumpState {
    normal: Option<Vec2>,
    ticks: u32,
}

fn reset_jump_state(
    mut controllers: Query<(
        &CharacterController,
        &CharacterIntent,
        &GroundNormal,
        &mut JumpState,
    )>,
) {
    for (controller, intent, ground_normal, mut jump_state) in &mut controllers {
        if !intent.jump
            && ground_normal.is_grounded()
            && jump_state.ticks >= controller.jump_min_ticks
        {
            jump_state.normal = None;
            jump_state.ticks = 0;
        }
    }
}

fn update_grounded(mut controllers: Query<(&CharacterController, &ShapeHits, &mut GroundNormal)>) {
    for (controller, hits, mut ground_norm) in &mut controllers {
        ground_norm.0 = hits
            .iter()
            .find(|hit| hit.normal1.angle_to(Vec2::Y).abs() < controller.max_slope_angle)
            .map(|hit| hit.normal1);
    }
}

fn apply_gravity(
    time: Res<Time>,
    gravity: Res<Gravity>,
    mut query: Query<(&GroundNormal, &mut LinearVelocity), With<CharacterController>>,
) {
    let g = gravity.0 * time.delta_secs();
    for (ground_normal, mut velocity) in &mut query {
        if !ground_normal.is_grounded() {
            velocity.0 += g;
        }
    }
}

fn apply_movement_damping(
    time: Res<Time>,
    mut query: Query<(&CharacterController, &GroundNormal, &mut LinearVelocity)>,
) {
    let dt = time.delta_secs();
    for (controller, ground_norm, mut velocity) in &mut query {
        let damping = if ground_norm.is_grounded() {
            controller.damping_ground
        } else {
            controller.damping_air
        };
        velocity.x *= 1.0 / (1.0 + damping * dt);
    }
}

fn apply_intents(
    time: Res<Time>,
    mut intents: Query<(
        &CharacterIntent,
        &CharacterController,
        &GroundNormal,
        &mut LinearVelocity,
        &mut JumpState,
    )>,
) {
    for (intent, controller, ground_norm, mut velocity, mut jump_state) in &mut intents {
        if let Some(normal) = ground_norm.0 {
            // Ground
            let accel = if intent.movement == 0.0 {
                controller.decel_ground
            } else {
                controller.accel_ground
            };

            let dv = accel * time.delta_secs();
            let cur_speed = velocity.x;
            let req_speed = intent.movement * controller.max_speed;

            let diff = req_speed - cur_speed;

            // Clamp acceleration
            if (diff / dv).abs() < 1.0 {
                velocity.x = req_speed;
            } else {
                velocity.x += diff.signum() * dv;
            }

            // Start jumping
            if intent.jump && jump_state.ticks == 0 {
                jump_state.normal = Some(normal);
            }
        } else {
            // Air
            velocity.x += intent.movement * controller.accel_air * time.delta_secs();
        }

        // Apply jump impulse for at least `jump_min_ticks` and at most `jump_max_ticks`.
        if jump_state.ticks < controller.jump_max_ticks
            && (intent.jump || jump_state.ticks < controller.jump_min_ticks)
            && let Some(normal) = jump_state.normal
        {
            velocity.0 += time.delta_secs() * controller.jump_impulse * normal;
            jump_state.ticks += 1;
        } else {
            jump_state.normal = None;
        }
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct MoveAndSlideResult(Option<MoveAndSlideOutput>);

fn handle_collisions(
    time: Res<Time>,
    // This parameter queries `Position`, so we can't update it in the same system.
    move_and_slide: MoveAndSlide,
    mut controllers: Query<
        (
            Entity,
            &Collider,
            &Rotation,
            &Position,
            &LinearVelocity,
            &mut MoveAndSlideResult,
        ),
        With<CustomPositionIntegration>,
    >,
) {
    for (entity, collider, rotation, position, velocity, mut result) in &mut controllers {
        if velocity.0 == Vec2::ZERO {
            continue;
        }

        let filter = SpatialQueryFilter::from_excluded_entities([entity]);
        let out = move_and_slide.move_and_slide(
            collider,
            position.0,
            rotation.as_radians(),
            velocity.0,
            time.delta(),
            &MoveAndSlideConfig::default(),
            &filter,
            |_hit| {
                // collisions.insert(hit.entity);
                MoveAndSlideHitResponse::Accept
            },
        );
        result.0 = Some(out);
    }
}

fn apply_move_and_slide(
    mut controllers: Query<(&mut MoveAndSlideResult, &mut Position, &mut LinearVelocity)>,
) {
    for (mut result, mut position, mut velocity) in &mut controllers {
        if let Some(out) = result.0.take() {
            position.0 = out.position;
            velocity.0 = out.projected_velocity;
        }
    }
}
