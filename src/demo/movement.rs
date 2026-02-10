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

use avian2d::prelude::{
    Collider, Forces, ShapeCastConfig, SpatialQuery, SpatialQueryFilter, WriteRigidBodyForces,
};
use bevy::prelude::*;

use crate::PausableSystems;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (update_grounded, apply_movement).in_set(PausableSystems),
    );
}

/// These are the movement parameters for our character controller.
/// For now, this is only used for a single player, but it could power NPCs or
/// other players as well.
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(OnGround)]
pub struct MovementController {
    /// The direction the character wants to move in.
    pub intent: Vec2,

    /// Whether the character is trying to jump.
    pub jump: bool,

    /// Maximum speed in world units per second.
    /// 1 world unit = 1 meter.
    pub max_speed: f32,
    pub air_speed: f32,
    pub jump_strength: f32,
    pub foot_offset: Vec2,
    pub foot_width: f32,

    pub has_been_grounded: bool,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            intent: Vec2::ZERO,
            jump: false,
            max_speed: 1.0,
            air_speed: 0.1,
            jump_strength: 15.,
            // For a 1.8m tall entity
            foot_offset: Vec2::new(0.0, -0.9),
            foot_width: 0.6,
            has_been_grounded: true,
        }
    }
}

#[derive(Component, Reflect, Default, Deref, Clone, Copy, PartialEq, Eq)]
#[reflect(Component)]
pub struct OnGround(bool);

fn update_grounded(
    spatial_query: SpatialQuery,
    mut controllers: Query<(Entity, &GlobalTransform, &MovementController, &mut OnGround)>,
) {
    for (entity, transform, controller, mut on_ground) in &mut controllers {
        on_ground.set_if_neq(OnGround(
            spatial_query
                .cast_shape(
                    &Collider::rectangle(controller.foot_width, 0.02),
                    transform.translation().xy() + controller.foot_offset,
                    0.0,
                    Dir2::NEG_Y,
                    &ShapeCastConfig {
                        max_distance: 0.01,
                        ..default()
                    },
                    &SpatialQueryFilter::from_excluded_entities([entity]),
                )
                .is_some(),
        ));
    }
}

fn apply_movement(mut movement_query: Query<(&mut MovementController, Ref<OnGround>, Forces)>) {
    for (mut controller, on_ground, mut forces) in &mut movement_query {
        let velocity = if on_ground.0 {
            controller.max_speed
        } else {
            controller.air_speed
        } * controller.intent;
        forces.apply_local_linear_impulse(velocity);

        if on_ground.0 {
            if on_ground.is_changed() {
                controller.has_been_grounded = true;
            }

            if controller.jump && controller.has_been_grounded {
                forces.apply_local_linear_impulse(controller.jump_strength * Vec2::Y);
                controller.has_been_grounded = false;
            }
        }
    }
}
