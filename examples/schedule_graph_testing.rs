#![allow(unused)]
use std::path::PathBuf;

use bevy::{prelude::*, render::RenderApp, utils::HashSet};
use bevy_ecs::schedule::{NodeId, ScheduleLabel};
use bevy_mod_debugdump::schedule_graph::Settings;

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

    // app.configure_set(TestSet::A.in_base_set(CoreSet::Update))
    // .add_systems((test_system_1, test_system_2).chain().in_set(TestSet::A));
    // app.configure_sets((TestSet::A, TestSet::B).chain().in_set(TestSet::C));
    app.add_plugins(DefaultPlugins.build().disable::<bevy::log::LogPlugin>());

    app.world
        .resource_scope::<Schedules, _>(|world, mut schedules| {
            let schedule_label = Main;
            let schedule = schedules.get_mut(&schedule_label).unwrap();

            schedule.graph_mut().initialize(world);
            schedule
                .graph_mut()
                .build_schedule(world.components())
                .unwrap();

            let settings = Settings {
                collapse_single_system_sets: true,
                ..Default::default()
            };
            let dot =
                bevy_mod_debugdump::schedule_graph::schedule_graph_dot(schedule, world, &settings);

            println!("{dot}");
        });

    Ok(())
}

fn print_schedule(schedule: &Schedule, schedule_label: &dyn ScheduleLabel) {
    let graph = schedule.graph();

    let name_of_node = |id: NodeId| {
        let name = match id {
            NodeId::System(_) => graph.system_at(id).name(),
            NodeId::Set(_) => {
                let name = format!("{:?}", graph.set_at(id));
                if let Some(name) = name.strip_prefix("SystemTypeSet(\"") {
                    let system_name = name.trim_end_matches("\")").to_string();
                    format!("@{system_name}").into()
                } else {
                    name.into()
                }
            }
        };
        if false {
            pretty_type_name::pretty_type_name_str(&name)
        } else {
            name.into()
        }
    };

    println!("{:?}", schedule_label);

    println!("- SETS");
    for (_set_id, set, _conditions) in graph.system_sets() {
        println!("  - {:?}", set);
    }

    println!("- SYSTEMS");
    for (_system_id, system, _conditions) in graph.systems() {
        println!("  - {}", system.name());
    }

    println!("- HIERARCHY");
    let hierarchy = graph.hierarchy();
    for (from, to, ()) in hierarchy.graph().all_edges() {
        println!("  - {} -> {}", name_of_node(from), name_of_node(to));
    }

    println!("- DEPENDENCY");
    let hierarchy = graph.dependency();
    for (from, to, ()) in hierarchy.graph().all_edges() {
        println!("  - {} -> {}", name_of_node(from), name_of_node(to));
    }

    println!();
}
