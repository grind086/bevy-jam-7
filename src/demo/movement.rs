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

use avian2d::prelude::{Collisions, Forces, LinearVelocity, WriteRigidBodyForces};
use bevy::prelude::*;

use crate::PausableSystems;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (update_grounded, apply_movement, apply_movement_damping).in_set(PausableSystems),
    );
}

/// These are the movement parameters for our character controller.
/// For now, this is only used for a single player, but it could power NPCs or
/// other players as well.
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(MovementIntent, OnGround)]
pub struct MovementController {
    /// Maximum speed in world units per second.
    /// 1 world unit = 1 meter.
    pub max_speed: f32,
    pub air_speed: f32,
    pub jump_strength: f32,
    pub damping_factor: f32,

    pub has_been_grounded: bool,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            max_speed: 1.0,
            air_speed: 0.1,
            jump_strength: 20.,
            damping_factor: 0.9,
            has_been_grounded: true,
        }
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct MovementIntent {
    pub direction: f32,
    pub jump: bool,
}

#[derive(Component, Reflect, Default, Deref, Clone, Copy, PartialEq, Eq)]
#[reflect(Component)]
pub struct OnGround(bool);

#[derive(Component, Reflect)]
#[reflect(Component)]
#[relationship_target(relationship = FootSensorOf)]
pub struct FootSensor(Entity);

#[derive(Component, Reflect)]
#[reflect(Component)]
#[relationship(relationship_target = FootSensor)]
pub struct FootSensorOf(pub Entity);

fn update_grounded(
    collisions: Collisions,
    mut controllers: Query<(Entity, &FootSensor, &mut OnGround)>,
) {
    for (entity, foot_sensor, mut on_ground) in &mut controllers {
        on_ground.set_if_neq(OnGround(
            collisions
                .entities_colliding_with(foot_sensor.0)
                .find(|e| *e != entity)
                .is_some(),
        ));
    }
}

fn apply_movement(
    mut movement_query: Query<(
        &MovementIntent,
        &mut MovementController,
        Ref<OnGround>,
        Forces,
    )>,
) {
    for (intent, mut controller, on_ground, mut forces) in &mut movement_query {
        let velocity = if on_ground.0 {
            controller.max_speed
        } else {
            controller.air_speed
        } * intent.direction;
        forces.apply_local_linear_impulse(velocity * Vec2::X);

        if on_ground.0 {
            if on_ground.is_changed() {
                controller.has_been_grounded = true;
            }

            if intent.jump && controller.has_been_grounded {
                forces.apply_local_linear_impulse(controller.jump_strength * Vec2::Y);
                controller.has_been_grounded = false;
            }
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
