//! Where things are and how they move.

use bevy_ecs::prelude::*;

use crate::Vec2;
use crate::determinism::Seq;
use crate::time::Time;

/// Position in the world. Every Transform automatically gets a PrevTransform and a Seq
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
#[require(PrevTransform, Seq)]
pub struct Transform {
    pub pos: Vec2,
}

impl Transform {
    pub fn at(x: f32, y: f32) -> Self {
        Self { pos: Vec2::new(x, y) }
    }
}

/// Where the entity was at the start of the current tick; the renderer
/// draws between this and Transform. None until the first tick.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub struct PrevTransform(pub Option<Vec2>);

/// Units per second.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub struct Velocity {
    pub v: Vec2,
}

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Self { v: Vec2::new(x, y) }
    }
}

/// System: move everything that has a Velocity by one tick.
pub fn integrate(time: Res<Time>, mut movers: Query<(&mut Transform, &Velocity)>) {
    let dt = time.dt as f32;

    for (mut t, v) in &mut movers {
        t.pos += v.v * dt;
    }
}

/// System (Decide, first): remember where everything is before it moves.
pub fn store_previous(mut q: Query<(&Transform, &mut PrevTransform)>) {
    for (t, mut prev) in &mut q {
        prev.0 = Some(t.pos);
    }
}
