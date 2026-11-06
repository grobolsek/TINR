//! Part 1: the basic engine, tested from the outside like a game would use it.

use bevy_ecs::prelude::*;
use tinr_engine::time::Time;

#[test]
fn world_holds_time() {
    let mut world = World::new();
    world.insert_resource(Time::new(60));

    // Change it through a mutable borrow...
    for _ in 0..60 {
        world.resource_mut::<Time>().advance_tick();
    }
    // ...and read it through a shared one.
    assert_eq!(world.resource::<Time>().now, 1.0);

    // A missing resource is an Option, not a crash, when you ask with get_resource.
    #[derive(Resource)]
    struct Score(u32);
    assert!(world.get_resource::<Score>().is_none());
    world.insert_resource(Score(3));
    assert_eq!(world.resource::<Score>().0, 3);
}

#[test]
fn entities_are_rows_and_queries_pick_columns() {
    use tinr_engine::transform::{Transform, Velocity};

    let mut world = World::new();
    let player = world.spawn((Transform::at(0.0, 0.0), Velocity::new(100.0, 0.0))).id();
    let rocket = world.spawn((Transform::at(10.0, 0.0), Velocity::new(0.0, -50.0))).id();
    let wall = world.spawn(Transform::at(50.0, 50.0)).id(); // no Velocity: never moves

    // Move everything that has both components, by hand, for half a second.
    let mut movers = world.query::<(&mut Transform, &Velocity)>();
    for (mut t, vel) in movers.iter_mut(&mut world) {
        t.pos += vel.v * 0.5;
    }

    assert_eq!(world.get::<Transform>(player).unwrap().pos.x, 50.0);
    assert_eq!(world.get::<Transform>(rocket).unwrap().pos.y, -25.0);
    assert_eq!(world.get::<Transform>(wall).unwrap().pos, tinr_engine::Vec2::new(50.0, 50.0));

    // Filters: entities with a Transform but without a Velocity.
    let mut still = world.query_filtered::<Entity, (With<Transform>, Without<Velocity>)>();
    assert_eq!(still.iter(&world).collect::<Vec<_>>(), vec![wall]);
}

#[test]
fn a_schedule_runs_systems_in_order() {
    use tinr_engine::transform::{Transform, Velocity, integrate};

    #[derive(Resource, Default)]
    struct Log(Vec<String>);

    fn before(mut log: ResMut<Log>) {
        log.0.push("before".into());
    }
    fn after(mut log: ResMut<Log>, movers: Query<&Transform>) {
        log.0.push(format!("after: {} moved", movers.iter().count()));
    }

    let mut world = World::new();
    world.insert_resource(Time::new(60));
    world.init_resource::<Log>();
    let e = world.spawn((Transform::default(), Velocity::new(60.0, 0.0))).id();

    let mut schedule = Schedule::default();
    schedule.add_systems((before, integrate, after).chain());

    for _ in 0..60 {
        world.resource_mut::<Time>().advance_tick();
        schedule.run(&mut world);
    }
    assert!((world.get::<Transform>(e).unwrap().pos.x - 60.0).abs() < 1e-3);
    assert_eq!(world.resource::<Log>().0[..2], ["before", "after: 1 moved"]);
}

#[test]
fn the_app_runs_one_phase_at_a_time() {
    use tinr_engine::app::{App, phase};
    use tinr_engine::transform::{Transform, Velocity, integrate};

    let mut app = App::new(60);
    app.add_systems(phase::Simulate, integrate);
    let e = app.world.spawn((Transform::default(), Velocity::new(60.0, 0.0))).id();

    app.world.resource_mut::<Time>().advance_tick();
    app.world.run_schedule(phase::Decide); // nothing registered there yet
    assert_eq!(app.world.get::<Transform>(e).unwrap().pos.x, 0.0);
    app.world.run_schedule(phase::Simulate);
    assert_eq!(app.world.get::<Transform>(e).unwrap().pos.x, 1.0);
}

#[test]
fn frames_at_144_fps_run_a_tick_every_second_or_third_frame() {
    use tinr_engine::app::App;

    let mut app = App::new(60);
    let mut ticks = Vec::new();
    let mut alphas = Vec::new();
    for _ in 0..7 {
        ticks.push(app.frame(1.0 / 144.0));
        alphas.push((app.world.resource::<Time>().alpha * 100.0).round() / 100.0);
    }
    assert_eq!(ticks, [0, 0, 1, 0, 1, 0, 0]);
    assert_eq!(alphas, [0.42, 0.83, 0.25, 0.67, 0.08, 0.5, 0.92]);
}

#[test]
fn a_long_freeze_runs_at_most_five_ticks_and_pause_runs_none() {
    use tinr_engine::app::App;

    let mut app = App::new(60);
    assert_eq!(app.frame(3.0), 5); // 3 s would be 180 ticks; capped to 0.25 s, then to 5
    assert_eq!(app.frame(1.0 / 60.0), 1); // the rest was dropped, not carried over

    app.world.resource_mut::<Time>().paused = true;
    assert_eq!(app.frame(1.0 / 60.0), 0);
    assert_eq!(app.world.resource::<Time>().tick, 6);
}
