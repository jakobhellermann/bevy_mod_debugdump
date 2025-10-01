#![allow(unused)]
use std::{collections::BTreeSet, path::PathBuf};

use bevy::{prelude::*, render::RenderApp};
use bevy_ecs::{
    component::ComponentId,
    schedule::{NodeId, ScheduleLabel},
};
use bevy_mod_debugdump::schedule_graph::Settings;
use bevy_platform::collections::hash_set::HashSet;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct ScheduleDebugGroup;

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

    app.world_mut()
        .resource_scope::<Schedules, _>(|world, mut schedules| {
            let ignored_ambiguities = schedules.ignored_scheduling_ambiguities.clone();
            let schedule_label = Main;
            let schedule = schedules.get_mut(schedule_label).unwrap();

            schedule.graph_mut().initialize(world);
            schedule
                .graph_mut()
                .build_schedule(world, &ignored_ambiguities)
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
            NodeId::System(id) => graph.systems.get(id).unwrap().name(),
            NodeId::Set(id) => {
                let name = format!("{:?}", graph.system_sets.get(id).unwrap());
                if let Some(name) = name.strip_prefix("SystemTypeSet(\"") {
                    let system_name = name.trim_end_matches("\")").to_string();
                    format!("@{system_name}").into()
                } else {
                    name.into()
                }
            }
        };
        if false {
            disqualified::ShortName(&name).to_string()
        } else {
            name.to_string()
        }
    };

    println!("{schedule_label:?}");

    println!("- SETS");
    for (_set_id, set, _conditions) in graph.system_sets.iter() {
        println!("  - {set:?}");
    }

    println!("- SYSTEMS");
    for (_system_id, system, _conditions) in graph.systems.iter() {
        println!("  - {}", system.name());
    }

    println!("- HIERARCHY");
    let hierarchy = graph.hierarchy();
    for (from, to) in hierarchy.graph().all_edges() {
        println!("  - {} -> {}", name_of_node(from), name_of_node(to));
    }

    println!("- DEPENDENCY");
    let hierarchy = graph.dependency();
    for (from, to) in hierarchy.graph().all_edges() {
        println!("  - {} -> {}", name_of_node(from), name_of_node(to));
    }

    println!();
}
