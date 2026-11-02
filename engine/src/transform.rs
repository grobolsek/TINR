//! Where things are and how they move.

use bevy_ecs::prelude::*;

use crate::Vec2;
use crate::time::Time;

/// Position in the world.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub struct Transform {
    pub pos: Vec2,
}

impl Transform {
    pub fn at(x: f32, y: f32) -> Self {
        Self { pos: Vec2::new(x, y) }
    }
}

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
