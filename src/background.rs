use bevy::{
    camera::ScalingMode,
    image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{AsBindGroup, encase::private::ShaderType},
    sprite_render::{Material2d, Material2dPlugin},
};

use crate::{asset_tracking::LoadResource, demo::player::PlayerCamera, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(Material2dPlugin::<ParallaxMaterial>::default());

    app.load_resource::<BackgroundAssets>()
        .add_systems(OnEnter(Screen::Gameplay), spawn_background)
        .add_systems(
            PostUpdate,
            (
                update_background_scale.before(TransformSystems::Propagate),
                update_background_material.after(TransformSystems::Propagate),
            ),
        );
}

#[derive(Resource, Asset, Reflect, Clone)]
#[reflect(Resource)]
struct BackgroundAssets {
    mesh: Handle<Mesh>,
    material: Handle<ParallaxMaterial>,
}

impl FromWorld for BackgroundAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        let material = ParallaxMaterial {
            scale: Vec2::splat(1. / 16.),
            offset: Vec2::new(0.0, 13.0),
            camera_position: Vec2::ZERO,
            back: assets.load_with_settings(
                "images/background/back-trees.png",
                |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                        address_mode_u: ImageAddressMode::Repeat,
                        ..ImageSamplerDescriptor::nearest()
                    });
                },
            ),
            middle: assets.load_with_settings(
                "images/background/middle-trees.png",
                |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                        address_mode_u: ImageAddressMode::Repeat,
                        ..ImageSamplerDescriptor::nearest()
                    });
                },
            ),
            front: assets.load_with_settings(
                "images/background/front-trees.png",
                |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                        address_mode_u: ImageAddressMode::Repeat,
                        ..ImageSamplerDescriptor::nearest()
                    });
                },
            ),
            light: assets.load_with_settings(
                "images/background/lights.png",
                |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                        address_mode_u: ImageAddressMode::Repeat,
                        ..ImageSamplerDescriptor::nearest()
                    });
                },
            ),
        };

        let mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(Rectangle::from_size(Vec2::ONE));

        let material = world
            .resource_mut::<Assets<ParallaxMaterial>>()
            .add(material);

        Self { mesh, material }
    }
}

#[derive(Component, Reflect)]
struct Background;

#[derive(AsBindGroup, Asset, Reflect, Clone)]
#[uniform(0, ParallaxUniforms)]
pub struct ParallaxMaterial {
    scale: Vec2,
    offset: Vec2,
    camera_position: Vec2,
    #[texture(1)]
    #[sampler(2)]
    back: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    middle: Handle<Image>,
    #[texture(5)]
    #[sampler(6)]
    front: Handle<Image>,
    #[texture(7)]
    #[sampler(8)]
    light: Handle<Image>,
}

impl Material2d for ParallaxMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "shaders/parallax.wgsl".into()
    }
}

#[derive(ShaderType)]
struct ParallaxUniforms {
    scale: Vec2,
    offset: Vec2,
    camera_position: Vec2,
}

impl From<&ParallaxMaterial> for ParallaxUniforms {
    fn from(value: &ParallaxMaterial) -> Self {
        Self {
            scale: value.scale,
            offset: value.offset,
            camera_position: value.camera_position,
        }
    }
}

fn spawn_background(
    assets: Res<BackgroundAssets>,
    camera: Single<Entity, With<PlayerCamera>>,
    mut commands: Commands,
) {
    commands.entity(camera.into_inner()).with_child((
        Name::new("Background"),
        Background,
        DespawnOnExit(Screen::Gameplay),
        GlobalZIndex(-1),
        Transform::default(),
        Mesh2d(assets.mesh.clone()),
        MeshMaterial2d(assets.material.clone()),
    ));
}

fn update_background_scale(
    camera: Single<&Projection, With<PlayerCamera>>,
    mut background: Single<&mut Transform, With<Background>>,
) {
    if let Projection::Orthographic(proj) = camera.into_inner()
        && let ScalingMode::Fixed { width, height } = proj.scaling_mode
    {
        let size = Vec2::new(width, height) / 32.;
        background.scale = size.extend(background.scale.z);
    };
}

fn update_background_material(
    camera: Single<&GlobalTransform, With<PlayerCamera>>,
    background: Single<&MeshMaterial2d<ParallaxMaterial>, With<Background>>,
    mut materials: ResMut<Assets<ParallaxMaterial>>,
) {
    if let Some(material) = materials.get_mut(&background.0) {
        material.camera_position = camera.translation().xy();
    }
}
