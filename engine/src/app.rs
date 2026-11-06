//! The App owns the World and runs the phases.

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::{ScheduleLabel, SingleThreadedExecutor};
use bevy_ecs::system::ScheduleSystem;

use crate::time::Time;

/// One schedule per phase of the frame. A system belongs to exactly one phase.
pub mod phase {
    use bevy_ecs::schedule::ScheduleLabel;

    /// Once, before the first frame: load the level.
    #[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Startup;
    /// Every frame, first: read input.
    #[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct First;
    /// Every tick: what does everyone want? (input, AI)
    #[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Decide;
    /// Every tick: move things.
    #[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Simulate;
    /// Every tick: find and resolve contacts.
    #[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Collide;
    /// Every tick: timers, effects, rules.
    #[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Gameplay;
    /// Every tick, last: rebuild derived data. Called "Sync" in the design;
    /// renamed because `Sync` is a standard Rust trait.
    #[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SyncPhase;
    /// Every frame: animation, camera, UI.
    #[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Update;
    /// Every frame: draw.
    #[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Render;
    /// Every frame, last: audio.
    #[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Last;
}

pub struct App {
    pub world: World,
    /// Real time not yet simulated, in seconds.
    accumulator: f64,
    /// At most this many ticks per frame; a slower machine drops time instead.
    pub max_ticks_per_frame: u32,
    started: bool,
}

impl App {
    pub fn new(fixed_hz: u32) -> Self {
        let mut world = World::new();
        world.insert_resource(Time::new(fixed_hz));

        add_phase(&mut world, phase::Startup);
        add_phase(&mut world, phase::First);
        add_phase(&mut world, phase::Decide);
        add_phase(&mut world, phase::Simulate);
        add_phase(&mut world, phase::Collide);
        add_phase(&mut world, phase::Gameplay);
        add_phase(&mut world, phase::SyncPhase);
        add_phase(&mut world, phase::Update);
        add_phase(&mut world, phase::Render);
        add_phase(&mut world, phase::Last);

        Self { world, accumulator: 0.0, max_ticks_per_frame: 5, started: false }
    }

    /// `app.add_systems(phase::Simulate, integrate)`, or several: `(a, b).chain()`.
    pub fn add_systems<M>(
        &mut self,
        phase: impl ScheduleLabel,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        self.world.resource_mut::<Schedules>().add_systems(phase, systems);
        self
    }

    /// One rendered frame: 0 to 5 fixed ticks, then the per-frame phases. Returns how many ticks ran.
    pub fn frame(&mut self, real_dt: f64) -> u32 {
        self.startup();

        let real_dt = real_dt.min(0.25);

        let (fixed_dt, paused, scale) = {
            let mut t = self.world.resource_mut::<Time>();
            t.frame += 1;
            t.real_dt = real_dt;
            (t.fixed_dt, t.paused, t.scale)
        };

        self.world.run_schedule(phase::First);

        if !paused {
            self.accumulator += real_dt * scale;
        }

        let mut ticks = 0;

        while self.accumulator >= fixed_dt - 1e-9 && ticks < self.max_ticks_per_frame {
            self.tick();
            self.accumulator -= fixed_dt;
            ticks += 1;
        }
        if ticks == self.max_ticks_per_frame && self.accumulator > fixed_dt {
            self.accumulator = 0.0; // too slow: drop time instead of spiralling
        }
        self.world.resource_mut::<Time>().alpha = (self.accumulator / fixed_dt).clamp(0.0, 1.0);

        self.world.run_schedule(phase::Update);
        self.world.run_schedule(phase::Render);
        self.world.run_schedule(phase::Last);
        ticks
    }

    /// One fixed step of the simulation: advance time, then run the tick phases in order.
    pub fn tick(&mut self) {
        self.world.resource_mut::<Time>().advance_tick();
        self.world.run_schedule(phase::Decide);
        self.world.run_schedule(phase::Simulate);
        self.world.run_schedule(phase::Collide);
        self.world.run_schedule(phase::Gameplay);
        self.world.run_schedule(phase::SyncPhase);
    }

    /// Runs Startup phase, once
    pub fn startup(&mut self) {
        if !self.started {
            self.world.run_schedule(phase::Startup);
            self.started = true;
        }
    }
}

/// Creates an empty phase. The single-threaded executor runs systems one after
/// another in a fixed order, which keeps every run identical.
fn add_phase(world: &mut World, label: impl ScheduleLabel) {
    let mut schedule = Schedule::new(label);
    schedule.set_executor(SingleThreadedExecutor::new());
    world.add_schedule(schedule);
}
