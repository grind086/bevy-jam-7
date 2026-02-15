//! Development tools for the game. This plugin is only enabled in dev builds.

use avian2d::prelude::{PhysicsDebugPlugin, PhysicsGizmos};
use bevy::{
    dev_tools::states::log_transitions,
    input::common_conditions::{input_just_pressed, input_toggle_active},
    prelude::*,
};
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin,
    quick::{
        AssetInspectorPlugin, FilterQueryInspectorPlugin, ResourceInspectorPlugin,
        WorldInspectorPlugin,
    },
};

use crate::{
    background::ParallaxMaterial,
    demo::{level::EnemyHandle, player::Player},
    physics::SpeedOfLight,
    screens::Screen,
};

const INSPECTOR_TOGGLE_KEY: KeyCode = KeyCode::Backquote;
const UI_DEBUG_TOGGLE_KEY: KeyCode = KeyCode::F1;
const PHYSICS_DEBUG_TOGGLE_KEY: KeyCode = KeyCode::F2;
const DESPAWN_ENEMIES_KEY: KeyCode = KeyCode::F12;

pub(super) fn plugin(app: &mut App) {
    // World inspector
    app.add_plugins((
        EguiPlugin::default(),
        WorldInspectorPlugin::default().run_if(input_toggle_active(true, INSPECTOR_TOGGLE_KEY)),
        ResourceInspectorPlugin::<SpeedOfLight>::new()
            .run_if(input_toggle_active(true, INSPECTOR_TOGGLE_KEY)),
        AssetInspectorPlugin::<ParallaxMaterial>::new()
            .run_if(input_toggle_active(true, INSPECTOR_TOGGLE_KEY)),
        FilterQueryInspectorPlugin::<With<Player>>::new()
            .run_if(input_toggle_active(true, INSPECTOR_TOGGLE_KEY)),
    ));

    // Physics
    app.add_plugins(PhysicsDebugPlugin)
        .insert_gizmo_config(
            PhysicsGizmos {
                axis_lengths: None,
                ..default()
            },
            GizmoConfig::default(),
        )
        .add_systems(
            Update,
            toggle_physics_gizmos.run_if(input_just_pressed(PHYSICS_DEBUG_TOGGLE_KEY)),
        );

    // Log `Screen` state transitions.
    app.add_systems(Update, log_transitions::<Screen>);

    // Toggle the debug overlay for UI.
    app.add_systems(
        Update,
        toggle_debug_ui.run_if(input_just_pressed(UI_DEBUG_TOGGLE_KEY)),
    );

    // Kill all enemies
    app.add_systems(
        Update,
        despawn_all_enemies.run_if(input_just_pressed(DESPAWN_ENEMIES_KEY)),
    );
}

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}

fn toggle_physics_gizmos(mut store: ResMut<GizmoConfigStore>) {
    let (config, _) = store.config_mut::<PhysicsGizmos>();
    config.enabled = !config.enabled;
}

fn despawn_all_enemies(enemies: Query<Entity, With<EnemyHandle>>, mut commands: Commands) {
    for enemy in &enemies {
        commands.entity(enemy).try_despawn();
    }
}
