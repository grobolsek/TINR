use tinr_engine::Vec2;
use tinr_engine::app::{App, phase};
use tinr_engine::bevy_ecs::prelude::*;
use tinr_engine::render::Sprite;
use tinr_engine::transform::{Transform, Velocity, integrate, store_previous};
use tinr_platform::{Config, run};

fn main() {
    let config = Config {
        title: tinr_engine::NAME.to_string(),
        // game/assets/atlas.png, found no matter which folder you run from.
        atlas: Some(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/atlas.png").to_string()),
        ..Default::default()
    };
    run(config, |app| {
        setup(app);
    });
}

fn setup(app: &mut App) {
    app.add_systems(phase::Decide, store_previous);
    app.add_systems(phase::Simulate, (integrate, bounce).chain());

    let boxes = [
        (Vec2::new(-200.0, -100.0), Vec2::new(150.0, 90.0), [230, 90, 80]),
        (Vec2::new(100.0, 50.0), Vec2::new(-120.0, 160.0), [90, 200, 120]),
        (Vec2::new(0.0, 150.0), Vec2::new(200.0, -60.0), [80, 150, 240]),
    ];
    for (pos, vel, color) in boxes {
        app.world.spawn((Transform { pos }, Velocity { v: vel }, Sprite::rect(30.0, 30.0, color)));
    }

    app.world.spawn((Transform::at(-100.0, 0.0), sprite(atlas::PLAYER_STATIC)));
    app.world.spawn((Transform::at(100.0, 0.0), sprite(atlas::ENEMY_STATIC)));
}

/// Sprite regions in game/assets/atlas.png, copied by hand from atlas.json.
/// Every sprite is a full 32x32 cell (packed with --trim-mode none --no-allow-rotation).
/// Re-pack → update these numbers. Part 3 replaces this with a loader.
#[allow(dead_code)]
mod atlas {
    pub const ENEMY_MOVING_1: f32 = 0.0;
    pub const ENEMY_MOVING_2: f32 = 32.0;
    pub const ENEMY_STATIC: f32 = 64.0;
    pub const PLAYER_JUMP: f32 = 96.0;
    pub const PLAYER_MOVING: f32 = 128.0;
    pub const PLAYER_STATIC: f32 = 160.0;
    pub const SPEAR: f32 = 192.0;
    pub const SWORD_SWING: f32 = 224.0;
    pub const SWORD: f32 = 256.0;
}

/// Pixels on screen per atlas pixel. Keep it a whole number for crisp pixel art.
const SCALE: f32 = 4.0;

/// A 32x32 sprite from the atlas strip, starting at pixel column `x`, drawn SCALE times bigger.
fn sprite(x: f32) -> Sprite {
    let mut s = Sprite::atlas(x, 0.0, 32.0, 32.0);
    s.size *= SCALE;
    s.layer = 1;
    s
}

/// System (Simulate, after integrate): keep everything inside the window.
fn bounce(mut q: Query<(&mut Transform, &mut Velocity)>) {
    let half = Vec2::new(385.0, 285.0);
    for (mut t, mut vel) in &mut q {
        if t.pos.x.abs() > half.x {
            vel.v.x = -vel.v.x;
            t.pos.x = t.pos.x.clamp(-half.x, half.x);
        }
        if t.pos.y.abs() > half.y {
            vel.v.y = -vel.v.y;
            t.pos.y = t.pos.y.clamp(-half.y, half.y);
        }
    }
}
