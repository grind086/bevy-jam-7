use avian2d::prelude::Collider;
use bevy::{
    math::{IRect, IVec2, URect, UVec2},
    prelude::Deref,
    reflect::{Reflect, ReflectDeserialize, ReflectSerialize},
    transform::components::Transform,
};
use serde::{Deserialize, Serialize};

/// A rectangle describing a collision rectangle for level terrain.
#[derive(Reflect, Serialize, Deserialize, Debug, Deref, Clone, Copy)]
#[reflect(Serialize, Deserialize)]
#[serde(transparent)]
pub struct LevelCollider(pub URect);

impl LevelCollider {
    /// Creates a collider and transform for this collider. These should be added as children of
    /// the collider.
    pub fn into_collider(self) -> (Collider, Transform) {
        let rect = self.as_rect();
        let size = rect.size();
        let center = rect.center();
        (
            Collider::rectangle(size.x, size.y),
            Transform::from_translation(center.extend(0.0)),
        )
    }
}

/// Used to build colliders from a boolean collision grid.
pub struct LevelCollisionBuilder {
    bounds: IRect,
    size: IVec2,
    collision_grid: Vec<bool>,
}

impl LevelCollisionBuilder {
    fn new(level_bounds: IRect, default: bool) -> Self {
        let level_size = level_bounds.size();
        Self {
            bounds: level_bounds,
            size: level_size,
            collision_grid: vec![default; level_size.element_product() as _],
        }
    }

    pub fn from_grid(size: UVec2, collision_grid: Vec<bool>) -> Self {
        assert_eq!(size.element_product() as usize, collision_grid.len());
        let size = size.as_ivec2();
        Self {
            bounds: IRect {
                min: IVec2::ZERO,
                max: size,
            },
            size,
            collision_grid,
        }
    }

    /// Creates a new grid with the given bounds and every cell set to `false`.
    pub fn new_empty(bounds: IRect) -> Self {
        Self::new(bounds, false)
    }

    /// Creates a new grid with the given bounds and every cell set to `true`.
    pub fn new_filled(bounds: IRect) -> Self {
        Self::new(bounds, true)
    }

    /// Sets the collision at the given grid coordinate.
    pub fn set(&mut self, grid: IVec2, collides: bool) -> &mut Self {
        if let Some(i) = self.linearize(grid) {
            self.collision_grid[i] = collides;
        }
        self
    }

    /// Sets the collision for multiple grid coordinates using an iterator.
    pub fn set_iter(&mut self, iter: impl IntoIterator<Item = (IVec2, bool)>) -> &mut Self {
        iter.into_iter().for_each(|(tile, collides)| {
            self.set(tile, collides);
        });
        self
    }

    /// Returns the collision at the given grid coordinate. Coordinates outside the grid return
    /// `false`.
    pub fn get(&self, grid: IVec2) -> bool {
        self.linearize(grid)
            .map_or(false, |i| self.collision_grid[i])
    }

    /// Builds a reduced set of rectangles from the current tile collision grid, calling
    /// `push_rect` for each collider rectangle produced.
    ///
    /// Rectangles are in world grid coordinates.
    // Inspired by: https://github.com/Trouv/bevy_ecs_ldtk/blob/d91241b8ca37f71d874398ee4c77b1b4bc782ff5/examples/platformer/walls.rs#L32
    fn build_rects(&self, mut push_rect: impl FnMut(IRect)) {
        let mut strips = Vec::with_capacity(self.bounds.height() as _);

        // Create one tile high strips of continuous collision areas.
        for y in self.bounds.min.y..self.bounds.max.y {
            let mut row_strips = Vec::new();
            let mut strip_start = None;

            // Collision is only counted in bounds, so going 1 past the left edge forces pending
            // strips to finish.
            for x in self.bounds.min.x..self.bounds.max.x + 1 {
                match (strip_start, self.get(IVec2 { x, y })) {
                    (None, true) => strip_start = Some(x),
                    (Some(left), false) => {
                        strip_start = None;
                        row_strips.push((left, x - 1));
                    }
                    _ => {}
                }
            }

            strips.push(row_strips);
        }

        // Add an empty row so that rectangles finish.
        strips.push(vec![]);

        // Vertically merge equal strips into rectangles.
        for row in 0..strips.len() {
            let (head, rest) = strips[row..].split_first_mut().unwrap();

            'outer: while let Some(strip) = head.pop() {
                for (dy, next_row) in rest.iter_mut().enumerate() {
                    if let Some(i) = next_row.iter().position(|next_strip| *next_strip == strip) {
                        // Strip exists in next row. Remove it so we don't produce overlapping duplicates.
                        next_row.remove(i);
                    } else {
                        // Strip doesn't exist in next row. Push the current rectangle and continue.
                        let y0 = self.bounds.min.y + row as i32;
                        let y1 = y0 + dy as i32 + 1;
                        push_rect(IRect {
                            min: IVec2::new(strip.0, y0),
                            max: IVec2::new(strip.1, y1),
                        });
                        continue 'outer;
                    };
                }
            }
        }
    }

    /// Builds a reduced set of rectangular [`LevelCollider`]s from the current collision grid, calling
    /// `push_collider` for each collider produced.
    pub fn build(&self) -> Vec<LevelCollider> {
        let mut colliders = Vec::new();

        self.build_rects(|rect| {
            colliders.push(LevelCollider(URect {
                min: (rect.min - self.bounds.min).as_uvec2(),
                max: (rect.max - self.bounds.max).as_uvec2(),
            }));
        });

        colliders
    }

    /// Returns the index of `grid` within `collision_grid`. Returns `None` if the coordinate is
    /// out of bounds.
    fn linearize(&self, grid: IVec2) -> Option<usize> {
        (grid.cmpge(self.bounds.min).all() && grid.cmplt(self.bounds.max).all())
            .then(|| {
                let local = grid - self.bounds.min;
                local.x + self.size.x * local.y
            })
            .map(|i| i as _)
    }
}
