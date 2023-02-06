use std::path::PathBuf;

use bevy::{prelude::*, render::RenderApp, utils::HashSet};
use bevy_mod_debugdump_stageless::Settings;

fn test_system_1() {}
fn test_system_2() {}
fn test_system_3() {}

#[derive(SystemSet, PartialEq, Eq, Clone, Hash, Debug)]
enum TestSet {
    A,
    B,
    C,
}

fn main() -> Result<(), std::io::Error> {
    let mut app = App::new();

    app.configure_set(TestSet::A.in_base_set(CoreSet::Update))
        .add_systems((test_system_1, test_system_2).chain().in_set(TestSet::A));
    // app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>());

    app.world
        .resource_scope::<Schedules, _>(|world, mut schedules| {
            let schedule_label = CoreSchedule::Main;
            let schedule = schedules.get_mut(&schedule_label).unwrap();

            schedule.graph_mut().initialize(world);
            schedule
                .graph_mut()
                .build_schedule(world.components())
                .unwrap();

            let settings = Settings::default();
            let dot = bevy_mod_debugdump_stageless::schedule_to_dot(schedule, world, &settings);

            println!("{dot}");
        });

    Ok(())
}
