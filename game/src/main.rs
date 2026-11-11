use tinr_engine::Vec2;
use tinr_engine::app::{App, phase};
use tinr_engine::bevy_ecs::prelude::*;
use tinr_engine::determinism::Rng;
use tinr_engine::input::RawInput;
use tinr_engine::lifetime::{Lifetime, tick_lifetimes};
use tinr_engine::render::Sprite;
use tinr_engine::state::{AppState, OnEnter, in_state};
use tinr_engine::time::Time;
use tinr_engine::transform::{Transform, Velocity, integrate, store_previous};
use tinr_platform::{Config, run};

fn main() {
    let config = Config { title: tinr_engine::NAME.to_string(), ..Default::default() };
    run(config, setup);
}

fn setup(app: &mut App) {
    app.add_systems(phase::Startup, spawn_boxes);
    app.add_systems(phase::First, global_keys);
    app.add_systems(phase::Decide, store_previous);
    app.add_systems(phase::Simulate, (integrate, bounce).chain());
    app.add_systems(phase::Gameplay, (spawn_sparks.run_if(in_state("Playing")), tick_lifetimes));
    app.add_systems(OnEnter("GameOver".into()), clear_movers);
    app.add_systems(OnEnter("Playing".into()), spawn_boxes);
}

/// System (Startup, and on entering Playing): three bouncing boxes.
fn spawn_boxes(mut commands: Commands) {
    let boxes = [
        (Vec2::new(-200.0, -100.0), Vec2::new(150.0, 90.0), [230, 90, 80]),
        (Vec2::new(100.0, 50.0), Vec2::new(-120.0, 160.0), [90, 200, 120]),
        (Vec2::new(0.0, 150.0), Vec2::new(200.0, -60.0), [80, 150, 240]),
    ];
    for (pos, v, color) in boxes {
        commands.spawn((Transform { pos }, Velocity { v }, Sprite::rect(30.0, 30.0, color)));
    }
}

/// System (First, every frame, even while paused): P pauses, T toggles slow
/// motion, G ends the game, R restarts it.
fn global_keys(input: Res<RawInput>, mut time: ResMut<Time>, mut state: ResMut<AppState>) {
    if input.just_pressed("p") {
        time.paused = !time.paused;
    }
    if input.just_pressed("t") {
        time.scale = if time.scale == 1.0 { 0.25 } else { 1.0 };
    }
    if input.just_pressed("g") {
        state.set("GameOver");
    }
    if input.just_pressed("r") {
        state.set("Playing");
    }
}

/// System (on entering GameOver): remove everything that moves.
fn clear_movers(movers: Query<Entity, With<Velocity>>, mut commands: Commands) {
    for e in &movers {
        commands.entity(e).despawn();
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

/// System (Gameplay, only while Playing): every 10 ticks, shoot a small spark
/// out of the center in a random direction.
fn spawn_sparks(time: Res<Time>, mut rng: ResMut<Rng>, mut commands: Commands) {
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
