use avian2d::prelude::{CollisionLayers, PhysicsLayer};

#[derive(PhysicsLayer, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamePhysicsLayers {
    #[default]
    LevelGeometry,
    Player,
}

pub trait GamePhysicsLayersExt {
    fn level_geometry() -> Self;
    fn player() -> Self;
}

impl GamePhysicsLayersExt for CollisionLayers {
    fn level_geometry() -> Self {
        CollisionLayers::new(GamePhysicsLayers::LevelGeometry, GamePhysicsLayers::Player)
    }

    fn player() -> Self {
        CollisionLayers::new(GamePhysicsLayers::Player, GamePhysicsLayers::LevelGeometry)
    }
}
