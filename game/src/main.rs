mod temp;

use tinr_engine::Vec2;
use tinr_engine::app::{App, phase};
use tinr_engine::bevy_ecs::prelude::*;
use tinr_engine::transform::{Transform, Velocity, integrate, store_previous};
use tinr_platform::{Config, run};

use temp::{Animation, Follow, Frame, animate, atlas, follow, frame, sprite};

fn main() {
    let config = Config {
        title: tinr_engine::NAME.to_string(),
        atlas: Some(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/atlas.png").to_string()),
        ..Default::default()
    };
    run(config, |app| {
        setup(app);
    });
}

/// Enemy walk: two frames, a quarter second each.
const ENEMY_WALK: &[Frame] =
    &[frame(atlas::ENEMY_MOVING_1, 0.25), frame(atlas::ENEMY_MOVING_2, 0.25)];

/// Sword: held for 0.75 s, then a quick swing. Same total (1 s) as two walk cycles,
/// so body and sword stay in step.
/// `.at(12.0, 0.0)` draws the swing 12 atlas pixels to the right; tune these two numbers.
const SWORD_ATTACK: &[Frame] =
    &[frame(atlas::SWORD, 0.75), frame(atlas::SWORD_SWING, 0.1).at(12.0, 0.0)];

fn setup(app: &mut App) {
    app.add_systems(phase::Decide, store_previous);
    // Move, keep inside the window, then put followers on their (already moved) targets.
    app.add_systems(phase::Simulate, (integrate, move_player, follow).chain());
    app.add_systems(phase::Update, animate);

    let enemy = app
        .world
        .spawn((
            Transform::at(-100.0, 0.0),
            Velocity::new(20.0, 0.0),
            sprite(atlas::ENEMY_STATIC),
            Animation::new(ENEMY_WALK),
        ))
        .id();

    // The sword has no Velocity: Follow moves it. Layer 2 draws it over the enemy.
    let mut sword = sprite(atlas::SWORD);
    sword.layer = 2;
    app.world.spawn((
        Transform::at(-100.0, 0.0),
        sword,
        Animation::new(SWORD_ATTACK),
        Follow::new(enemy),
    ));
}

/// System (Simulate, after integrate): keep everything inside the window.
fn move_player(mut q: Query<(&mut Transform, &mut Velocity)>) {
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
