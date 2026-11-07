//! The platform layer: window, frame loop and drawing, with Macroquad.
//! Swapping Macroquad for something else touches only this crate.

use macroquad::prelude::*;
use tinr_engine::app::{App, phase};
use tinr_engine::bevy_ecs::prelude::*;
use tinr_engine::render::{Shape, Sprite};
use tinr_engine::time::Time;
use tinr_engine::transform::{PrevTransform, Transform};

pub struct Config {
    pub title: String,
    pub window_size: (i32, i32),
    pub fixed_hz: u32,
    /// Path to the sprite atlas PNG. Sprites with a `region` are cut from it.
    pub atlas: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self { title: "TINR".into(), window_size: (800, 600), fixed_hz: 60, atlas: None }
    }
}

/// Opens a window, builds the App with `setup`, and runs one `app.frame()` per screen frame.
//
/// Command line: `--frames N` stops after N frames of exactly 1/60 s,
/// `--screenshot out.png` saves the last frame.
pub fn run(config: Config, setup: impl FnOnce(&mut App) + 'static) {
    let Config { title, window_size: (w, h), fixed_hz, atlas } = config;
    let args = Args::parse();
    let conf = Conf {
        window_title: title.to_string(),
        window_width: w,
        window_height: h,
        ..Default::default()
    };

    macroquad::Window::from_config(conf, async move {
        if let Some(path) = &atlas {
            match load_texture(path).await {
                Ok(tex) => {
                    tex.set_filter(FilterMode::Nearest); // crisp pixel art
                    ATLAS.with(|a| *a.borrow_mut() = Some(tex));
                }
                Err(e) => eprintln!("could not load atlas {path}: {e}"),
            }
        }

        let mut app = App::new(fixed_hz);
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

// The atlas texture. Kept outside the World for now; Macroquad draws on one thread.
thread_local! {
    static ATLAS: std::cell::RefCell<Option<Texture2D>> = const { std::cell::RefCell::new(None) };
}

struct Args {
    frames: Option<u32>,
    screenshot: Option<String>,
}

impl Args {
    fn parse() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let value =
            |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned();
        Self {
            frames: value("--frames").and_then(|v| v.parse().ok()),
            screenshot: value("--screenshot"),
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
        let p = from.lerp(t.pos, alpha) + center + sprite.offset;
        let s = sprite.size;
        if let Some([x, y, w, h]) = sprite.region {
            let drawn = ATLAS.with(|a| {
                let Some(tex) = &*a.borrow() else { return false };
                let params = DrawTextureParams {
                    dest_size: Some(vec2(s.x, s.y)),
                    source: Some(Rect::new(x, y, w, h)),
                    ..Default::default()
                };
                draw_texture_ex(tex, p.x - s.x / 2.0, p.y - s.y / 2.0, WHITE, params);
                true
            });
            if drawn {
                continue;
            }
            // No atlas loaded: fall through and draw the colored shape instead.
        }
        let [r, g, b] = sprite.color;
        let color = Color::from_rgba(r, g, b, 255);
        match sprite.shape {
            Shape::Circle => draw_circle(p.x, p.y, sprite.radius, color),
            Shape::Rect => {
                draw_rectangle(p.x - s.x / 2.0, p.y - s.y / 2.0, s.x, s.y, color);
            }
        }
    }
}
