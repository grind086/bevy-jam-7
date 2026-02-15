use avian2d::{
    PhysicsPlugins,
    physics_transform::PhysicsTransformSystems,
    prelude::{LinearVelocity, PhysicsSystems},
};
use bevy::{camera::ScalingMode, prelude::*, window::PrimaryWindow};

use crate::{
    controller::CharacterController,
    demo::{
        level::LevelGeometry,
        player::{Player, PlayerCamera},
    },
};

mod layers;

pub use layers::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(PhysicsPlugins::default())
        .insert_resource(SpeedOfLight(25.0));

    app.add_systems(
        FixedPostUpdate,
        (
            (update_level_length_contraction, update_length_contraction)
                .before(PhysicsTransformSystems::Propagate),
            update_lorentz_factors.in_set(PhysicsSystems::StepSimulation),
        ),
    );
}

// TODO: Either refactor this to actually just be player vs level geometry using resources, or
// actually figure out how to do collision for (non-physical) multi-target scaling. One collider
// for the level frame and one (+sprite) for the player frame, with a proper frame "ghost"?

#[derive(Resource, Reflect, Deref, Clone, Copy, PartialEq, PartialOrd)]
#[reflect(Resource)]
pub struct SpeedOfLight(pub f32);

impl Default for SpeedOfLight {
    fn default() -> Self {
        Self(299_792_458.0)
    }
}

#[derive(Component, Reflect)]
pub struct LorentzFactor(pub Vec2);

impl Default for LorentzFactor {
    fn default() -> Self {
        Self(Vec2::ONE)
    }
}

fn gamma(s: f32, c: f32) -> f32 {
    let b = s.abs().min(c * 0.999) / c;
    1.0 / (1.0 - b * b).sqrt()
}

fn update_lorentz_factors(
    time: Res<Time>,
    c: Res<SpeedOfLight>,
    player_vel: Single<&LinearVelocity, With<Player>>,
    mut velocities: Query<(&LinearVelocity, &mut LorentzFactor)>,
) {
    for (target_vel, mut lorentz) in &mut velocities {
        let v = player_vel.0 - target_vel.0;
        let g = Vec2::new(gamma(v.x, c.0), gamma(v.y, c.0));
        lorentz.0 = lorentz.0.lerp(g, (4.0 * time.delta_secs()).min(1.0));

        let should_round = (lorentz.0 - 1.0).cmplt(Vec2::splat(0.001));
        if should_round.y {
            lorentz.0.y = 1.0;
        }
        if should_round.x {
            lorentz.0.x = 1.0;
        }
    }
}

fn update_level_length_contraction(
    gamma: Single<&LorentzFactor, With<LevelGeometry>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<&mut Projection, With<PlayerCamera>>,
    mut player: Single<(&mut Transform, &mut CharacterController), With<Player>>,
) {
    let Projection::Orthographic(proj) = &mut *camera.into_inner() else {
        return;
    };

    let window_size = window.size() * gamma.0;
    proj.scaling_mode = ScalingMode::Fixed {
        width: window_size.x,
        height: window_size.y,
    };

    player.0.scale = gamma.0.extend(player.0.scale.z);
    player.1.max_speed = 20. * gamma.0.x;
    player.1.accel_air = 3.5 * gamma.0.x.sqrt();
    player.1.accel_ground = 35. * gamma.0.x.sqrt();
    // player.1.damping_factor_air = 0.3 * gamma.0.x.sqrt();
    // player.1.damping_factor_ground = 2.5 * gamma.0.x.sqrt();
}

fn update_length_contraction(
    mut transforms: Query<(&LorentzFactor, &mut Transform), Without<LevelGeometry>>,
) {
    for (gamma, mut local) in &mut transforms {
        local.scale = (1.0 / gamma.0).extend(local.scale.z);
    }
}
