//! App states (Playing, GameOver, Menu, ...): which systems run, and what
//! happens when the state changes.

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ScheduleLabel;

/// The current state, and the one requested for the next transition.
/// States are names so that data files can name them too.
#[derive(Resource, Debug, Clone)]
pub struct AppState {
    pub current: String,
    pub next: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self { current: "Playing".into(), next: None }
    }
}

impl AppState {
    /// Requests a change; it happens at the end of the current tick or frame.
    pub fn set(&mut self, name: &str) {
        self.next = Some(name.to_string());
    }
}

/// Runs once when a state is entered: `app.add_systems(OnEnter("GameOver".into()), ...)`.
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct OnEnter(pub String);

/// Runs once when a state is left.
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct OnExit(pub String);

/// Run condition: `my_system.run_if(in_state("Playing"))`.
pub fn in_state(name: &'static str) -> impl FnMut(Res<AppState>) -> bool + Clone {
    move |state: Res<AppState>| state.current == name
}

/// Applies a requested change: OnExit of the old state, then OnEnter of the new one.
pub fn apply_state_transition(world: &mut World) {
    let (old, new) = {
        let mut state = world.resource_mut::<AppState>();
        let Some(new) = state.next.take() else {
            return;
        };
        if new == state.current {
            return;
        }
        let old = std::mem::replace(&mut state.current, new.clone());
        (old, new)
    };
    // try_run_schedule: a state without OnExit/OnEnter systems has no schedule; that is fine.
    let _ = world.try_run_schedule(OnExit(old));
    let _ = world.try_run_schedule(OnEnter(new));
}
