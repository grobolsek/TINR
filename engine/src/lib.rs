//! The TINR engine: everything that is not a game.

/// The engine's name, used by the game's window title.
pub const NAME: &str = "FRI the ninja";

pub mod app;
pub mod render;
pub mod time;
pub mod transform;

pub use bevy_ecs;
pub use glam::Vec2;

#[cfg(test)]
mod tests {
    #[test]
    fn name_is_set() {
        assert_eq!(super::NAME, "TINR");
    }
}
