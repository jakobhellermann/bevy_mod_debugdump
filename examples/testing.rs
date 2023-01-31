use bevy::{
    log::LogPlugin,
    prelude::{System, World},
    DefaultPlugins,
};
use bevy_app::{App, CoreSchedule, PluginGroup};
use bevy_ecs::scheduling::{NodeId, Schedule, ScheduleLabel, Schedules};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>());

    let mut schedules = app.world.resource_mut::<Schedules>();

    let schedule_label = CoreSchedule::Main;
    let schedule = schedules.get_mut(&schedule_label).unwrap();

    let mut world = World::default();
    schedule.graph_mut().initialize(&mut world);
    schedule
        .graph_mut()
        .build_schedule(world.components())
        .unwrap();

    if true {
        let ignore_asset_event = |system: &dyn System<In = (), Out = ()>| {
            let name = system.name();
            let _ignore = ![
                "asset_event_system",
                ">::update_system",
                "update_asset_storage",
            ]
            .iter()
            .any(|ignore| name.contains(ignore));
            _ignore
            // true
        };
        let settings = bevy_mod_debugdump_stageless::Settings {
            show_single_system_in_set: true,
            include_system: Box::new(ignore_asset_event),
            ..Default::default()
        };
        let dot = bevy_mod_debugdump_stageless::schedule_to_dot(
            &schedule_label,
            schedule,
            &world,
            &settings,
        );
        println!("{dot}");
    } else if false {
        print_schedule(schedule, &schedule_label);
    }

    if false {
        let schedule = app.get_schedule_mut(schedule_label).unwrap();
        schedule
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
