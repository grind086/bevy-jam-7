//! Development tools for the game. This plugin is only enabled in dev builds.

// use avian2d::prelude::{PhysicsDebugPlugin, PhysicsGizmos};
use bevy::{
    dev_tools::states::log_transitions,
    input::common_conditions::{input_just_pressed, input_toggle_active},
    prelude::*,
};
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin,
    quick::{ResourceInspectorPlugin, WorldInspectorPlugin},
};

use crate::{physics::SpeedOfLight, screens::Screen};

const INSPECTOR_TOGGLE_KEY: KeyCode = KeyCode::Backquote;
const UI_DEBUG_TOGGLE_KEY: KeyCode = KeyCode::F1;

pub(super) fn plugin(app: &mut App) {
    // World inspector
    app.add_plugins((
        EguiPlugin::default(),
        WorldInspectorPlugin::default().run_if(input_toggle_active(true, INSPECTOR_TOGGLE_KEY)),
        ResourceInspectorPlugin::<SpeedOfLight>::new()
            .run_if(input_toggle_active(true, INSPECTOR_TOGGLE_KEY)),
    ));

    // Physics
    // app.add_plugins(PhysicsDebugPlugin).insert_gizmo_config(
    //     PhysicsGizmos {
    //         axis_lengths: None,
    //         ..default()
    //     },
    //     GizmoConfig::default(),
    // );

    // Log `Screen` state transitions.
    app.add_systems(Update, log_transitions::<Screen>);

    // Toggle the debug overlay for UI.
    app.add_systems(
        Update,
        toggle_debug_ui.run_if(input_just_pressed(UI_DEBUG_TOGGLE_KEY)),
    );
}

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}
