use tinr_engine::Vec2;
use tinr_engine::app::{App, phase};
use tinr_engine::bevy_ecs::prelude::*;
use tinr_engine::render::Sprite;
use tinr_engine::transform::{Transform, Velocity, integrate, store_previous};
use tinr_platform::{Config, run};

fn main() {
    run(Config { title: tinr_engine::NAME.to_string(), ..Default::default() }, |app| {
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
