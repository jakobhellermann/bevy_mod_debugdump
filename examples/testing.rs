use bevy::{log::LogPlugin, DefaultPlugins};
use bevy_app::{App, CoreSchedule, PluginGroup};
use bevy_ecs::scheduling::{NodeId, Schedule, ScheduleLabel, Schedules};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>());

    let schedules = app.world.resource::<Schedules>();

    let schedule_label = CoreSchedule::Main;
    let schedule = schedules.get(&schedule_label).unwrap();
    if true {
        let dot = bevy_mod_debugdump_stageless::schedule_to_dot(&schedule_label, schedule);
        println!("{dot}");
    } else {
        print_schedule(schedule, &schedule_label);
    }

    if false {
        app.get_schedule_mut(schedule_label)
            .unwrap()
            .initialize(&mut bevy_ecs::world::World::new())
            .unwrap();
    }
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
    for (from, to, ()) in hierarchy.graph.all_edges() {
        println!("  - {} -> {}", name_of_node(from), name_of_node(to));
    }

    println!("- DEPENDENCY");
    let hierarchy = graph.dependency();
    for (from, to, ()) in hierarchy.graph.all_edges() {
        println!("  - {} -> {}", name_of_node(from), name_of_node(to));
    }

    println!();
}
