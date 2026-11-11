//! Part 1: the basic engine, tested from the outside like a game would use it.

use bevy_ecs::prelude::*;
use tinr_engine::lifetime::{Lifetime, tick_lifetimes};
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

#[test]
fn every_transform_gets_a_previous_position_for_drawing() {
    use tinr_engine::app::{App, phase};
    use tinr_engine::transform::{PrevTransform, Transform, Velocity, integrate, store_previous};

    let mut app = App::new(60);
    app.add_systems(phase::Decide, store_previous);
    app.add_systems(phase::Simulate, integrate);
    // PrevTransform is not spawned here: Transform requires it, so bevy adds it.
    let e = app.world.spawn((Transform::at(0.0, 0.0), Velocity::new(60.0, 0.0))).id();
    assert_eq!(app.world.get::<PrevTransform>(e).unwrap().0, None);

    app.tick();
    // Decide stored the old position, then Simulate moved the entity.
    assert_eq!(app.world.get::<PrevTransform>(e).unwrap().0, Some(tinr_engine::Vec2::ZERO));
    assert_eq!(app.world.get::<Transform>(e).unwrap().pos.x, 1.0);
}

#[test]
fn structural_changes_wait_for_a_sync_point() {
    use tinr_engine::app::{App, phase};
    use tinr_engine::lifetime::Lifetime;

    #[derive(Resource, Default)]
    struct Seen(Vec<usize>);

    fn spawn_one(mut commands: Commands) {
        commands.spawn(Lifetime::secs(1.0));
    }
    fn count(q: Query<&Lifetime>, mut seen: ResMut<Seen>) {
        seen.0.push(q.iter().count());
    }

    let mut app = App::new(60);
    app.world.init_resource::<Seen>();
    // Ordered, but told not to sync: `count` runs before the spawn is applied.
    app.add_systems(phase::Decide, (spawn_one, count).chain_ignore_deferred());
    // A later phase: the end of Decide was a sync point.
    app.add_systems(phase::Simulate, count);
    // Ordered with .chain(): bevy inserts a sync point between the two.
    app.add_systems(phase::Gameplay, (spawn_one, count).chain());

    app.tick();
    // Decide's count saw nothing yet; Simulate saw the Decide spawn;
    // Gameplay's count saw both spawns because of the chained sync point.
    assert_eq!(app.world.resource::<Seen>().0, [0, 1, 2]);
}

#[test]
fn lifetimes_despawn_through_commands() {
    use tinr_engine::app::{App, phase};
    use tinr_engine::lifetime::{Lifetime, tick_lifetimes};

    let mut app = App::new(60);
    app.add_systems(phase::Gameplay, tick_lifetimes);
    let e = app.world.spawn(Lifetime::secs(0.04)).id(); // 2.4 ticks at 60 Hz

    app.tick();
    app.tick();
    assert!(app.world.get_entity(e).is_ok());
    app.tick();
    assert!(app.world.get_entity(e).is_err());
}

#[test]
fn the_same_seed_gives_the_same_numbers() {
    use tinr_engine::determinism::Rng;

    let draws = |seed| {
        let mut rng = Rng::new(seed);
        (0..5).map(|_| rng.range(0.0, 100.0).round()).collect::<Vec<_>>()
    };
    assert_eq!(draws(7), draws(7));
    assert_ne!(draws(7), draws(8));
}

#[test]
fn seq_is_spawn_order_even_when_bevy_reuses_slots() {
    use tinr_engine::app::App;
    use tinr_engine::determinism::Seq;
    use tinr_engine::transform::Transform;

    let mut app = App::new(60);
    let a = app.world.spawn(Transform::at(1.0, 0.0)).id();
    let b = app.world.spawn(Transform::at(2.0, 0.0)).id();
    app.world.despawn(a);
    let c = app.world.spawn(Transform::at(3.0, 0.0)).id(); // takes a's old slot

    let mut q = app.world.query::<(Entity, &Seq)>();
    let by_entity: Vec<Entity> = q.iter(&app.world).sort::<Entity>().map(|(e, _)| e).collect();
    let by_seq: Vec<Entity> = q.iter(&app.world).sort::<&Seq>().map(|(e, _)| e).collect();
    assert_eq!(by_entity, [c, b]); // slot order: c sits in the lower slot
    assert_eq!(by_seq, [b, c]); // spawn order
}

#[test]
fn two_runs_with_the_same_seed_end_identical() {
    use tinr_engine::app::{App, phase};
    use tinr_engine::determinism::{Rng, checksum};
    use tinr_engine::transform::{Transform, Velocity, integrate};

    fn spawn_random(mut rng: ResMut<Rng>, mut commands: Commands) {
        let v = tinr_engine::Vec2::new(rng.range_f32(-100.0, 100.0), rng.range_f32(-100.0, 100.0));
        commands.spawn((Transform::default(), Velocity { v }));
    }
    let run = |seed| {
        let mut app = App::with_seed(60, seed);
        app.add_systems(phase::Simulate, integrate);
        app.add_systems(phase::Gameplay, spawn_random);
        for _ in 0..120 {
            app.frame(1.0 / 60.0);
        }
        checksum(&mut app.world)
    };
    assert_eq!(run(5), run(5));
    assert_ne!(run(5), run(6));
}
