//! TEMPORARY: hand-written atlas regions, frame animation and "follow another entity".
//! Delete this file when the guide gets there:
//! - `Follow`              → Part 2, transforms and hierarchy (child entities)
//! - `atlas`, `Frame`      → Part 3, the asset server and data files
//! - `Animation`, `animate` → when animations become data

use tinr_engine::Vec2;
use tinr_engine::bevy_ecs::prelude::*;
use tinr_engine::render::Sprite;
use tinr_engine::time::Time;
use tinr_engine::transform::Transform;

/// Screen pixels per atlas pixel. Keep it a whole number for crisp pixel art.
pub const SCALE: f32 = 4.0;

/// Sprite regions in game/assets/atlas.png as [x, y, w, h], copied by hand from atlas.json.
/// Re-pack → copy the new `frame` numbers here. Any size works (e.g. a 64x64 swing).
#[allow(dead_code)]
pub mod atlas {
    pub type Region = [f32; 4];
    pub const ENEMY_MOVING_1: Region = [0.0, 0.0, 32.0, 32.0];
    pub const ENEMY_MOVING_2: Region = [32.0, 0.0, 32.0, 32.0];
    pub const ENEMY_STATIC: Region = [64.0, 0.0, 32.0, 32.0];
    pub const PLAYER_JUMP: Region = [96.0, 0.0, 32.0, 32.0];
    pub const PLAYER_MOVING: Region = [128.0, 0.0, 32.0, 32.0];
    pub const PLAYER_STATIC: Region = [160.0, 0.0, 32.0, 32.0];
    pub const SPEAR: Region = [192.0, 0.0, 32.0, 32.0];
    pub const SWORD_SWING: Region = [224.0, 0.0, 32.0, 32.0];
    pub const SWORD: Region = [256.0, 0.0, 32.0, 32.0];
}

/// A sprite showing one atlas region, drawn SCALE times bigger.
pub fn sprite(region: atlas::Region) -> Sprite {
    let [x, y, w, h] = region;
    let mut s = Sprite::atlas(x, y, w, h);
    s.size *= SCALE;
    s.layer = 1;
    s
}

/// One frame of an animation.
#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub region: atlas::Region,
    /// Seconds this frame is shown.
    pub secs: f32,
    /// Where it is drawn relative to the entity, in ATLAS pixels (+x right, +y down).
    pub offset: Vec2,
}

/// `frame(atlas::SWORD, 0.5)`; add `.at(dx, dy)` to draw it shifted.
pub const fn frame(region: atlas::Region, secs: f32) -> Frame {
    Frame { region, secs, offset: Vec2::ZERO }
}

impl Frame {
    /// Draw this frame `dx, dy` atlas pixels away from the entity (+x right, +y down).
    pub const fn at(mut self, dx: f32, dy: f32) -> Self {
        self.offset = Vec2::new(dx, dy);
        self
    }
}

/// Plays a list of frames in a loop. Replacing the component restarts it from frame 0.
#[derive(Component)]
pub struct Animation {
    pub frames: &'static [Frame],
    /// Seconds since the animation started.
    pub time: f32,
}

impl Animation {
    pub fn new(frames: &'static [Frame]) -> Self {
        Self { frames, time: 0.0 }
    }

    /// The frame showing at `self.time`.
    fn current(&self) -> Frame {
        let total: f32 = self.frames.iter().map(|f| f.secs).sum();
        if total <= 0.0 {
            return self.frames[0];
        }
        let mut t = self.time % total;
        for f in self.frames {
            if t < f.secs {
                return *f;
            }
            t -= f.secs;
        }
        self.frames[self.frames.len() - 1]
    }
}

/// System (Update): show the current frame of every animation.
/// Slow motion slows it, pause freezes it.
pub fn animate(time: Res<Time>, mut q: Query<(&mut Animation, &mut Sprite)>) {
    if time.paused {
        return;
    }
    let dt = (time.real_dt * time.scale) as f32;
    for (mut anim, mut sprite) in &mut q {
        anim.time += dt;
        let f = anim.current();
        let [_, _, w, h] = f.region;
        sprite.region = Some(f.region);
        sprite.size = Vec2::new(w, h) * SCALE;
        sprite.offset = f.offset * SCALE;
    }
}

/// Keeps this entity at `target`'s position plus `offset` (screen pixels).
/// Give it no Velocity. If the target is despawned, it just stops moving.
#[derive(Component)]
pub struct Follow {
    pub target: Entity,
    pub offset: Vec2,
}

impl Follow {
    pub fn new(target: Entity) -> Self {
        Self { target, offset: Vec2::ZERO }
    }
}

/// System (Simulate, after everything that moves things): put followers on their target.
pub fn follow(mut q: Query<(&Follow, &mut Transform)>, targets: Query<&Transform, Without<Follow>>) {
    for (f, mut t) in &mut q {
        if let Ok(target) = targets.get(f.target) {
            t.pos = target.pos + f.offset;
        }
    }
}
