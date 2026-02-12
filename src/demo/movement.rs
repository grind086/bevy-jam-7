//! Handle player input and translate it into movement through a character
//! controller. A character controller is the collection of systems that govern
//! the movement of characters.
//!
//! In our case, the character controller has the following logic:
//! - Set [`MovementController`] intent based on directional keyboard input.
//!   This is done in the `player` module, as it is specific to the player
//!   character.
//! - Apply movement based on [`MovementController`] intent and maximum speed.
//! - Wrap the character within the window.
//!
//! Note that the implementation used here is limited for demonstration
//! purposes. If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/main/examples/movement/physics_in_fixed_timestep.rs).

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{PausableSystems, physics::GamePhysicsLayers};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(FixedPreUpdate, update_grounded_caster_scales)
        .add_systems(
            FixedUpdate,
            (update_grounded, apply_movement, apply_movement_damping)
                .chain()
                .in_set(PausableSystems),
        );
}

pub fn movement_controller(
    config: MovementController,
    collider: Collider,
    offset: Vec2,
    layers: CollisionLayers,
) -> impl Bundle {
    (
        config,
        Mass(1.5),
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED,
        GroundNormal::default(),
        ShapeCaster::new(collider.clone(), offset, 0.0, Dir2::NEG_Y).with_query_filter(
            SpatialQueryFilter::from_mask(GamePhysicsLayers::LevelGeometry),
        ),
        children![(
            layers,
            collider,
            Transform::from_translation(offset.extend(0.0))
        )],
    )
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(MovementIntent, GroundNormal)]
pub struct MovementController {
    pub max_speed: f32,
    pub air_speed: f32,
    pub jump_strength: f32,
    pub damping_factor: f32,
    pub max_slope_angle: f32,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            max_speed: 1.0,
            air_speed: 0.1,
            jump_strength: 20.,
            damping_factor: 0.9,
            max_slope_angle: f32::to_radians(45.0),
        }
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct MovementIntent {
    pub direction: f32,
    pub jump: bool,
}

#[derive(Component, Reflect, Default, Deref, Clone, Copy, PartialEq)]
#[reflect(Component)]
pub struct GroundNormal(Option<Vec2>);

impl GroundNormal {
    pub fn is_grounded(&self) -> bool {
        self.0.is_some()
    }
}

fn update_grounded_caster_scales(
    mut query: Query<(&GlobalTransform, &mut ShapeCaster), With<MovementController>>,
) {
    for (transform, mut caster) in &mut query {
        caster.shape.set_scale(0.9 * transform.scale().xy(), 10);
        caster.max_distance = 0.1 * transform.scale().y;
    }
}

fn update_grounded(mut controllers: Query<(&MovementController, &ShapeHits, &mut GroundNormal)>) {
    for (controller, hits, mut ground_norm) in &mut controllers {
        ground_norm.0 = hits
            .iter()
            .find(|hit| hit.normal1.angle_to(Vec2::Y).abs() < controller.max_slope_angle)
            .map(|hit| hit.normal1);
    }
}

fn apply_movement(
    mut movement_query: Query<(&MovementIntent, &MovementController, &GroundNormal, Forces)>,
) {
    for (intent, controller, ground_norm, mut forces) in &mut movement_query {
        let speed = if ground_norm.is_grounded() {
            controller.max_speed
        } else {
            controller.air_speed
        };
        forces.apply_local_linear_impulse(speed * intent.direction * Vec2::X);

        if let Some(normal) = ground_norm.0
            && intent.jump
        {
            forces.apply_local_linear_impulse(controller.jump_strength * normal);
        }
    }
}

fn apply_movement_damping(
    time: Res<Time>,
    mut query: Query<(&MovementController, &mut LinearVelocity)>,
) {
    let dt = time.delta_secs();
    for (controller, mut linear_velocity) in &mut query {
        linear_velocity.x *= 1.0 / (1.0 + controller.damping_factor * dt);
    }
}
