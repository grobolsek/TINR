//! Same seed and same input give the same game: a seeded random number
//! generator, and a stable order for entities.

use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::prelude::*;
use bevy_ecs::world::DeferredWorld;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::{Rng as _, SeedableRng};

/// The only source of randomness in the simulation. Never use the system clock
/// or thread_rng() in gameplay code: a replay could not repeat it.
#[derive(Resource, Clone)]
pub struct Rng(ChaCha8Rng);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(ChaCha8Rng::seed_from_u64(seed))
    }

    /// A number in [0, 1).
    pub fn next_f64(&mut self) -> f64 {
        // The top 53 bits of a u64 fill an f64's mantissa exactly.
        (self.0.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// A number in [lo, hi).
    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.next_f64()
    }

    pub fn range_f32(&mut self, lo: f32, hi: f32) -> f32 {
        self.range(lo as f64, hi as f64) as f32
    }
}

/// An entity's place in spawn order. bevy reuses the slots of despawned
/// entities, so sorting by Entity is not spawn order; sorting by Seq is.
/// Systems whose result depends on order iterate with `.sort::<&Seq>()`.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[component(on_add = assign_seq)]
pub struct Seq(pub u64);

/// The last Seq handed out.
#[derive(Resource, Debug, Default)]
pub struct SeqCounter(pub u64);

/// Hook: runs when Seq is added. A Seq of 0 means "not numbered yet".
fn assign_seq(mut world: DeferredWorld, ctx: HookContext) {
    if world.get::<Seq>(ctx.entity).is_some_and(|s| s.0 != 0) {
        return; // already numbered (a loaded save keeps its numbers)
    }
    let Some(mut counter) = world.get_resource_mut::<SeqCounter>() else {
        return; // a bare World without the counter: leave it at 0
    };
    counter.0 += 1;
    let next = counter.0;
    if let Some(mut seq) = world.get_mut::<Seq>(ctx.entity) {
        seq.0 = next;
    }
}

/// A fingerprint of every position, in Seq order. Two runs that agree on it
/// agree on where everything is, to the last bit.
pub fn checksum(world: &mut World) -> u64 {
    use crate::transform::Transform;
    let mut q = world.query::<(&Seq, &Transform)>();
    let mut hash: u64 = 0xcbf29ce484222325; // FNV-1a
    for (seq, t) in q.iter(world).sort::<&Seq>() {
        for word in [seq.0, t.pos.x.to_bits() as u64, t.pos.y.to_bits() as u64] {
            hash ^= word;
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    hash
}
