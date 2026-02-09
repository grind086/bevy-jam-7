use avian2d::{
    PhysicsPlugins,
    prelude::{Forces, LinearVelocity, WriteRigidBodyForces},
};
use bevy::prelude::*;

use crate::{
    PausableSystems,
    demo::{movement::MovementController, player::Player},
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(PhysicsPlugins::default())
        .init_resource::<SpeedOfLight>();

    app.add_systems(
        FixedUpdate,
        (apply_movement, update_lorentz_factors)
            .chain()
            .in_set(PausableSystems),
    );
}

#[derive(Resource, Reflect, Deref, Clone, Copy, PartialEq, PartialOrd)]
#[reflect(Resource)]
pub struct SpeedOfLight(pub f32);

impl Default for SpeedOfLight {
    fn default() -> Self {
        Self(299_792_458.0)
    }
}

#[derive(Component, Reflect)]
pub struct LorentzFactor {
    scalar: f32,
    vector: Vec2,
}

impl LorentzFactor {
    const CLAMP_INFINITE: f32 = 1000.0;

    fn new(v: Vec2, c: SpeedOfLight) -> Self {
        let (dir, speed) = v.normalize_and_length();
        let b = speed.min(c.0) / c.0;
        let g = 1.0 / (1.0 - b.powi(2)).sqrt();
        let g = g.min(Self::CLAMP_INFINITE);
        Self {
            scalar: g,
            vector: g * dir,
        }
    }

    // pub fn is_finite(&self) -> bool {
    //     self.scalar.is_finite()
    // }

    // pub fn is_unit(&self) -> bool {
    //     self.scalar == 1.0
    // }

    // pub fn scalar(&self) -> f32 {
    //     self.scalar
    // }

    // pub fn vector(&self) -> Vec2 {
    //     self.vector
    // }
}

impl Default for LorentzFactor {
    fn default() -> Self {
        Self {
            scalar: 1.0,
            vector: Vec2::X,
        }
    }
}

fn apply_movement(mut movement_query: Query<(&MovementController, Forces)>) {
    for (controller, mut forces) in &mut movement_query {
        let velocity = controller.max_speed * controller.intent;
        forces.apply_local_linear_impulse(velocity);
    }
}

fn update_lorentz_factors(
    c: Res<SpeedOfLight>,
    player_vel: Single<&LinearVelocity, With<Player>>,
    mut velocities: Query<(&LinearVelocity, &mut LorentzFactor)>,
) {
    for (target_vel, mut gamma) in &mut velocities {
        let relative_vel = player_vel.0 - target_vel.0;
        *gamma = LorentzFactor::new(relative_vel, *c);
    }
}
