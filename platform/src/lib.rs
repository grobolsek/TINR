//! The platform layer: window, frame loop and drawing, with Macroquad.
//! Swapping Macroquad for something else touches only this crate.

use macroquad::prelude::*;
use tinr_engine::app::{App, phase};
use tinr_engine::bevy_ecs::prelude::*;
use tinr_engine::bevy_ecs::resource::IsResource;
use tinr_engine::determinism::checksum;
use tinr_engine::render::{Shape, Sprite};
use tinr_engine::time::Time;
use tinr_engine::transform::{PrevTransform, Transform};

pub struct Config {
    pub title: String,
    pub window_size: (i32, i32),
    pub fixed_hz: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self { title: "TINR".into(), window_size: (800, 600), fixed_hz: 60 }
    }
}

/// Opens a window, builds the App with `setup`, and runs one `app.frame()` per screen frame.
//
/// `--screenshot out.png` saves the last frame, `--seed S` seeds the Rng,
/// `--headless` runs without a window and prints a summary.
pub fn run(config: Config, setup: impl FnOnce(&mut App) + 'static) {
    let Config { title, window_size: (w, h), fixed_hz } = config;
    let args = Args::parse();
    if args.headless {
        return run_headless(&args, setup);
    }
    let conf = Conf {
        window_title: title.to_string(),
        window_width: w,
        window_height: h,
        ..Default::default()
    };

    macroquad::Window::from_config(conf, async move {
        let mut app = App::with_seed(fixed_hz, args.seed);
        app.add_systems(phase::Render, draw_sprites);
        setup(&mut app);

        let mut frames = 0;

        loop {
            let dt = if args.frames.is_some() { 1.0 / 60.0 } else { get_frame_time() as f64 };
            app.frame(dt);
            frames += 1;
            if Some(frames) == args.frames {
                if let Some(path) = &args.screenshot {
                    get_screen_data().export_png(path);
                }
                break;
            }
            next_frame().await;
        }
    });
}

/// The same frames, without a window: for tests, CI and checking determinism.
fn run_headless(args: &Args, setup: impl FnOnce(&mut App)) {
    let mut app = App::with_seed(60, args.seed);
    setup(&mut app);
    let frames = args.frames.unwrap_or(600);
    for _ in 0..frames {
        app.frame(1.0 / 60.0);
    }
    let ticks = app.world.resource::<Time>().tick;
    // bevy keeps each resource (Time, Rng, ...) as an entity marked IsResource; skip those.
    let entities =
        app.world.query_filtered::<Entity, Without<IsResource>>().iter(&app.world).count();
    let sum = checksum(&mut app.world);
    println!("{frames} frames, {ticks} ticks, {entities} entities, checksum {sum:016x}");
}

struct Args {
    frames: Option<u32>,
    screenshot: Option<String>,
    seed: u64,
    headless: bool,
}

impl Args {
    fn parse() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let value =
            |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned();
        Self {
            frames: value("--frames").and_then(|v| v.parse().ok()),
            screenshot: value("--screenshot"),
            seed: value("--seed").and_then(|v| v.parse().ok()).unwrap_or(1),
            headless: args.iter().any(|a| a == "--headless"),
        }
    }
}

/// System (Render): draw every sprite between its last two positions, lowest layer first.
fn draw_sprites(time: Res<Time>, sprites: Query<(&Transform, &PrevTransform, &Sprite)>) {
    clear_background(Color::from_rgba(30, 32, 40, 255));
    // World origin at the screen center.
    let center = vec2(screen_width(), screen_height()) / 2.0;
    let alpha = time.alpha as f32;

    let mut list: Vec<_> = sprites.iter().collect();
    list.sort_by_key(|(_, _, s)| s.layer);
    for (t, prev, sprite) in list {
        let from = prev.0.unwrap_or(t.pos);
        let p = from.lerp(t.pos, alpha) + center;
        let [r, g, b] = sprite.color;
        let color = Color::from_rgba(r, g, b, 255);
        match sprite.shape {
            Shape::Circle => draw_circle(p.x, p.y, sprite.radius, color),
            Shape::Rect => {
                let s = sprite.size;
                draw_rectangle(p.x - s.x / 2.0, p.y - s.y / 2.0, s.x, s.y, color);
            }
        }
    }
}
