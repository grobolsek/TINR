use bevy_ecs::prelude::*;
/// Plays a list of atlas columns in a loop.
#[derive(Component)]
pub struct Animation {
    pub frames: &'static [f32], // atlas columns, e.g. [atlas::PLAYER_STATIC, atlas::PLAYER_MOVING, ...]
    pub fps: f32,               // frames per second
    pub time: f32,              // seconds since the animation started
}

impl Animation {
    pub fn new(frames: &'static [f32], fps: f32) -> Self {
        Self { frames, fps, time: 0.0 }
    }
}
