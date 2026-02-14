use avian2d::prelude::Collider;
use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    platform::collections::HashMap,
    prelude::*,
};

use crate::{
    animation::Animation, assets::serialize::enemy as de, demo::movement::MovementController,
};

#[derive(Asset, Reflect, Debug)]
pub struct Enemy {
    pub name: String,
    pub size: Vec2,
    pub atlas: Handle<Image>,
    pub atlas_layout: Handle<TextureAtlasLayout>,
    pub idle_anim: Handle<Animation>,
    pub walk_anim: Handle<Animation>,
    pub jump_anim: Handle<Animation>,
    pub peak_anim: Handle<Animation>,
    pub fall_anim: Handle<Animation>,
    #[reflect(ignore)]
    pub collider: Collider,
    pub collider_offset: Vec2,
    pub movement: MovementController,
}

#[derive(Asset, Reflect)]
pub struct EnemyManifest {
    pub enemies: HashMap<String, Handle<Enemy>>,
}

#[derive(TypePath, Default)]
pub struct EnemyManifestLoader;

impl AssetLoader for EnemyManifestLoader {
    type Asset = EnemyManifest;
    type Settings = ();
    type Error = BevyError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        &(): &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let mut manifest = HashMap::new();
        let manifest_toml: de::EnemyManifest = serde_json::from_slice(&bytes)?;
        for (label, enemy_def) in manifest_toml.enemies {
            let handle = load_context.labeled_asset_scope(label.clone(), |ctx| {
                let enemy = Enemy {
                    name: enemy_def.name.clone(),
                    size: enemy_def.size,
                    atlas: ctx.load(enemy_def.atlas),
                    atlas_layout: ctx.add_labeled_asset(
                        format!("{label}_layout"),
                        TextureAtlasLayout::from_grid(
                            enemy_def.atlas_layout.size,
                            enemy_def.atlas_layout.cols,
                            enemy_def.atlas_layout.rows,
                            None,
                            None,
                        ),
                    ),
                    idle_anim: load_animation(ctx, &label, &enemy_def.atlas_animations, "idle")
                        .ok_or("missing idle animation")?,
                    walk_anim: load_animation(ctx, &label, &enemy_def.atlas_animations, "walk")
                        .ok_or("missing walk animation")?,
                    jump_anim: load_animation(ctx, &label, &enemy_def.atlas_animations, "jump")
                        .ok_or("missing jump animation")?,
                    peak_anim: load_animation(ctx, &label, &enemy_def.atlas_animations, "peak")
                        .ok_or("missing peak animation")?,
                    fall_anim: load_animation(ctx, &label, &enemy_def.atlas_animations, "fall")
                        .ok_or("missing fall animation")?,
                    collider: enemy_def.collider.shape.into(),
                    collider_offset: enemy_def.collider.offset,
                    movement: MovementController {
                        max_speed: enemy_def.movement.max_speed,
                        accel_air: enemy_def.movement.accel_air,
                        accel_ground: enemy_def.movement.accel_ground,
                        jump_strength: enemy_def.movement.jump_strength,
                        damping_factor_air: enemy_def.movement.damping_factor_air,
                        damping_factor_ground: enemy_def.movement.damping_factor_ground,
                        max_slope_angle: enemy_def.movement.max_slope_angle,
                    },
                };

                info!("Loaded enemy {label:?}");

                Ok::<_, BevyError>(enemy)
            })?;

            manifest.insert(label, handle);
        }

        Ok(EnemyManifest { enemies: manifest })
    }

    fn extensions(&self) -> &[&str] {
        &["toml"]
    }
}

fn load_animation(
    ctx: &mut LoadContext<'_>,
    label: &str,
    atlas_animations: &HashMap<String, de::EnemyAnimation>,
    name: &str,
) -> Option<Handle<Animation>> {
    atlas_animations.get(name).map(|anim| {
        ctx.add_labeled_asset(
            format!("{label}_{name}_anim"),
            Animation::from_frame_range_and_millis(anim.start..anim.end, anim.frame_millis.into()),
        )
    })
}
