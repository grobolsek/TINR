use tinr_engine::Vec2;
use tinr_engine::app::{App, phase};
use tinr_engine::bevy_ecs::prelude::*;
use tinr_engine::bevy_ecs::system::command;
use tinr_engine::determinism::Rng;
use tinr_engine::lifetime::{Lifetime, tick_lifetimes};
use tinr_engine::render::Sprite;
use tinr_engine::time::Time;
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
    app.add_systems(phase::Gameplay, (spawn_sparks, tick_lifetimes));

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

pub fn spawn_sparks(time: Res<Time>, mut rng: ResMut<Rng>, mut commands: Commands) {
    if !time.tick.is_multiple_of(10) {
        return;
    }
    let angle = rng.range_f32(0.0, std::f32::consts::TAU);
    let speed = rng.range_f32(80.0, 220.0);
    commands.spawn((
        Transform::default(),
        Velocity { v: Vec2::from_angle(angle) * speed },
        Lifetime::secs(2.0),
        Sprite::circle(5.0, [250, 210, 90]),
    ));
}
