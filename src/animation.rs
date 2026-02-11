use std::{ops::Range, time::Duration};

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<Animation>().add_systems(
        Update,
        (update_animation_players, update_sprite_animations).chain(),
    );
}

#[derive(EntityEvent)]
pub struct AnimationEvent {
    #[event_target]
    pub entity: Entity,
    pub marker: usize,
}

#[derive(Asset, Reflect)]
pub struct Animation {
    pub frames: Vec<Frame>,
}

impl Animation {
    pub fn from_frame_range_and_millis(range: Range<usize>, frame_millis: u64) -> Self {
        let duration = Duration::from_millis(frame_millis);
        Self {
            frames: range
                .map(|index| Frame {
                    index,
                    duration,
                    markers: Vec::new(),
                })
                .collect(),
        }
    }

    pub fn with_marker(mut self, marker: usize, frames: impl IntoIterator<Item = usize>) -> Self {
        for i in frames {
            self.frames[i].markers.push(marker);
        }
        self
    }
}

#[derive(Reflect)]
pub struct Frame {
    pub index: usize,
    pub duration: Duration,
    pub markers: Vec<usize>,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(AnimationPlayerState)]
pub struct AnimationPlayer {
    pub animation: Handle<Animation>,
    pub retain_state: bool,
}

impl From<Handle<Animation>> for AnimationPlayer {
    fn from(animation: Handle<Animation>) -> Self {
        Self {
            animation,
            retain_state: false,
        }
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct AnimationPlayerState {
    frame_index: usize,
    atlas_index: usize,
    timer: Timer,
}

impl AnimationPlayerState {
    // pub fn frame_index(&self) -> usize {
    //     self.frame_index
    // }

    // pub fn atlas_index(&self) -> usize {
    //     self.atlas_index
    // }

    fn init(animation: &Animation) -> Self {
        let Some(first_frame) = animation.frames.first() else {
            return Self::default();
        };

        Self {
            frame_index: 0,
            atlas_index: first_frame.index,
            timer: Timer::new(first_frame.duration, TimerMode::Once),
        }
    }

    fn tick(&mut self, delta: Duration) -> bool {
        self.timer.tick(delta).is_finished()
    }

    fn go_to_next_frame<'a>(&mut self, animation: &'a Animation) -> &'a [usize] {
        if animation.frames.is_empty() {
            return &[];
        }

        let index = (self.frame_index + 1) % animation.frames.len();
        let frame = &animation.frames[index];

        self.frame_index = index;
        self.atlas_index = frame.index;
        self.timer = Timer::new(frame.duration, TimerMode::Once);

        &frame.markers
    }
}

fn update_animation_players(
    time: Res<Time>,
    animations: Res<Assets<Animation>>,
    mut animation_players: Query<(Entity, Ref<AnimationPlayer>, &mut AnimationPlayerState)>,
    mut commands: Commands,
) {
    for (entity, player, mut state) in &mut animation_players {
        let Some(animation) = animations.get(&player.animation) else {
            continue;
        };

        if player.is_changed() && !player.retain_state {
            *state = AnimationPlayerState::init(animation);
            continue;
        }

        if state.bypass_change_detection().tick(time.delta()) {
            for &marker in state.go_to_next_frame(animation) {
                commands.trigger(AnimationEvent { entity, marker });
            }
        }
    }
}

fn update_sprite_animations(
    mut sprites: Query<(&mut Sprite, &AnimationPlayerState), Changed<AnimationPlayerState>>,
) {
    for (mut sprite, state) in &mut sprites {
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.index = state.atlas_index;
        }
    }
}
