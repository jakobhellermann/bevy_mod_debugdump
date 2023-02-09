use bevy::prelude::{IntoSystemConfig, SystemSet, World};
use bevy_app::{App, CoreSchedule};
use bevy_ecs::schedule::{NodeId, Schedule, ScheduleLabel, Schedules};

fn test_system() {}

#[derive(SystemSet, PartialEq, Eq, Clone, Hash, Debug)]
struct TestSet;

fn main() {
    let mut app = App::new();
    app.add_system(test_system.in_set(TestSet));
    // app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>());

    let mut schedules = app.world.resource_mut::<Schedules>();

    let schedule_label = CoreSchedule::Main;
    let schedule = schedules.get_mut(&schedule_label).unwrap();

    let mut world = World::default();
    schedule.graph_mut().initialize(&mut world);
    schedule
        .graph_mut()
        .build_schedule(world.components())
        .unwrap();

    print_schedule(schedule, &schedule_label);
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
    for (_set_id, set, _, _conditions) in graph.system_sets() {
        println!("  - {:?}", set);
    }

    println!("- SYSTEMS");
    for (_system_id, system, _, _conditions) in graph.systems() {
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
