//! Simulated time, stored in the World as a resource.

use bevy_ecs::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct Time {
    /// Fixed ticks run so far.
    pub tick: u64,
    /// Simulated seconds = tick * fixed_dt. Computed, never summed, so it does not drift.
    pub now: f64,
    /// Seconds per fixed tick (1/60).
    pub fixed_dt: f64,
    /// Step of the current tick (equals fixed_dt during a tick).
    pub dt: f64,
    /// Leftover accumulator as a fraction of fixed_dt; the renderer lerps with it.
    pub alpha: f64,
    /// Slow motion: multiplies the real time fed into the accumulator.
    pub scale: f64,
    /// Paused: no real time is fed into the accumulator.
    pub paused: bool,
    /// Rendered frames so far.
    pub frame: u64,
    /// The last frame's real duration in seconds (clamped).
    pub real_dt: f64,
}

impl Time {
    pub fn new(fixed_hz: u32) -> Self {
        Self {
            tick: 0,
            now: 0.0,
            fixed_dt: 1.0 / fixed_hz as f64,
            dt: 0.0,
            alpha: 0.0,
            scale: 0.0,
            paused: false,
            frame: 0,
            real_dt: 0.0,
        }
    }

    pub fn advance_tick(&mut self) {
        self.tick += 1;
        self.now = self.tick as f64 * self.fixed_dt;
        self.dt = self.fixed_dt;
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new(60)
    }
}
