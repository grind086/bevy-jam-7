use avian2d::prelude::Collider;
use bevy::{
    asset::AssetPath,
    math::{UVec2, Vec2},
    platform::collections::HashMap,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct EnemyManifest {
    pub enemies: HashMap<String, Enemy>,
}

#[derive(Serialize, Deserialize)]
pub struct Enemy {
    pub name: String,
    pub size: Vec2,
    pub atlas: AssetPath<'static>,
    pub atlas_layout: EnemyAtlasLayout,
    pub atlas_animations: HashMap<String, EnemyAnimation>,
    pub collider: EnemyCollider,
    pub movement: EnemyMovement,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnemyAtlasLayout {
    pub rows: u32,
    pub cols: u32,
    pub size: UVec2,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnemyAnimation {
    pub start: usize,
    pub end: usize,
    pub frame_millis: u32,
}

#[derive(Serialize, Deserialize)]
pub struct EnemyCollider {
    #[serde(flatten)]
    pub shape: ColliderShape,
    #[serde(default)]
    pub offset: Vec2,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "shape")]
pub enum ColliderShape {
    Rectangle { width: f32, height: f32 },
    Capsule { radius: f32, height: f32 },
}

impl From<ColliderShape> for Collider {
    fn from(value: ColliderShape) -> Self {
        match value {
            ColliderShape::Rectangle { width, height } => Collider::rectangle(width, height),
            ColliderShape::Capsule { radius, height } => Collider::capsule(radius, height),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct EnemyMovement {
    pub max_speed: f32,
    pub air_speed: f32,
    pub jump_strength: f32,
    pub damping_factor: f32,
    pub max_slope_angle: f32,
}

impl Default for EnemyMovement {
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
