use std::ops::BitOr;

use avian2d::prelude::{CollisionLayers, PhysicsLayer};

#[derive(PhysicsLayer, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamePhysicsLayers {
    #[default]
    LevelGeometry,
    Player,
    Enemy,
}

impl BitOr for GamePhysicsLayers {
    type Output = u32;
    fn bitor(self, rhs: Self) -> Self::Output {
        self.to_bits() | rhs.to_bits()
    }
}

use GamePhysicsLayers::*;

pub trait GamePhysicsLayersExt {
    fn level_geometry() -> Self;
    fn player() -> Self;
    fn enemy() -> Self;
}

impl GamePhysicsLayersExt for CollisionLayers {
    fn level_geometry() -> Self {
        CollisionLayers::new(LevelGeometry, Player | Enemy)
    }

    fn player() -> Self {
        CollisionLayers::new(Player, LevelGeometry | Enemy)
    }

    fn enemy() -> Self {
        CollisionLayers::new(Enemy, LevelGeometry | Player)
    }
}
