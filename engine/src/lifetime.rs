//! Entities that remove themselves after a while.

use bevy_ecs::prelude::*;

use crate::time::Time;

/// Seconds left before the entity is despawned.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Lifetime {
    pub remaining: f64,
}

impl Lifetime {
    pub fn secs(remaining: f64) -> Self {
        Self { remaining }
    }
}

/// System (Gameplay): count down; despawn at zero. The despawn is a command:
/// it is queued now and applied at the next sync point, so every other system
/// in this phase still sees the entity.
pub fn tick_lifetimes(
    time: Res<Time>,
    mut q: Query<(Entity, &mut Lifetime)>,
    mut commands: Commands,
) {
    for (entity, mut life) in &mut q {
        life.remaining -= time.dt;
        if life.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
