use macroquad::prelude::*;

#[macroquad::main("FRI the ninja")]
async fn main() {
    loop {
        clear_background(DARKGRAY);
        let text = format!("{} is alive", tinr_engine::NAME);
        draw_text(&text, 20.0, 40.0, 32.0, WHITE);
        // Hand the frame to the screen and wait for the next one.
        next_frame().await;
    }
}
