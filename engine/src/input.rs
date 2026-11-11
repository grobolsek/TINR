//! Raw input as the platform reports it: which keys are down this frame.
//! Key names are lowercase: "a", "space", "left", "f5", "mouse_left".

use std::collections::BTreeSet;

use bevy_ecs::prelude::*;

use crate::Vec2;

#[derive(Resource, Debug, Default, Clone)]
pub struct RawInput {
    /// Keys held down right now.
    pub down: BTreeSet<String>,
    /// Keys that went down since the last frame.
    pub pressed: BTreeSet<String>,
    /// Mouse position in world coordinates.
    pub mouse: Vec2,
}

impl RawInput {
    pub fn is_down(&self, key: &str) -> bool {
        self.down.contains(key)
    }

    pub fn just_pressed(&self, key: &str) -> bool {
        self.pressed.contains(key)
    }
}
