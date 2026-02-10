use avian2d::{
    PhysicsPlugins,
    physics_transform::PhysicsTransformSystems,
    prelude::{Forces, LinearVelocity, PhysicsSystems, WriteRigidBodyForces},
};
use bevy::{camera::ScalingMode, prelude::*, window::PrimaryWindow};

use crate::{
    PausableSystems,
    demo::{
        level::LevelGeometry,
        movement::MovementController,
        player::{Player, PlayerCamera},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(PhysicsPlugins::default())
        .insert_resource(SpeedOfLight(50.0));

    app.add_systems(FixedUpdate, apply_movement.in_set(PausableSystems))
        .add_systems(
            FixedPostUpdate,
            (
                (update_level_length_contraction, update_length_contraction)
                    .before(PhysicsTransformSystems::Propagate),
                update_lorentz_factors.in_set(PhysicsSystems::StepSimulation),
            ),
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
    const CLAMP_INFINITE: f32 = 100.0;

    fn new(v: Vec2, c: SpeedOfLight) -> Self {
        let (dir, speed) = v.normalize_and_length();
        let b = speed.min(c.0) / c.0;
        let g = 1.0 / (1.0 - b.powi(2)).sqrt();
        let g = g.clamp(1.0, Self::CLAMP_INFINITE);
        Self {
            scalar: g,
            vector: ((g - 1.) * dir).abs() + 1.,
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

    pub fn vector(&self) -> Vec2 {
        self.vector
    }
}

impl Default for LorentzFactor {
    fn default() -> Self {
        Self {
            scalar: 1.0,
            vector: Vec2::ONE,
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

fn update_level_length_contraction(
    gamma: Single<&LorentzFactor, With<LevelGeometry>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<&mut Projection, With<PlayerCamera>>,
    mut player: Single<&mut Transform, With<Player>>,
) {
    let Projection::Orthographic(proj) = &mut *camera.into_inner() else {
        return;
    };

    let window_size = window.size() * gamma.vector();
    proj.scaling_mode = ScalingMode::Fixed {
        width: window_size.x,
        height: window_size.y,
    };

    player.scale = gamma.vector().extend(player.scale.z);
}

fn update_length_contraction(
    mut transforms: Query<(&LorentzFactor, &mut Transform), Without<LevelGeometry>>,
) {
    for (gamma, mut local) in &mut transforms {
        local.scale = (1.0 / gamma.vector()).extend(local.scale.z);
    }
}
