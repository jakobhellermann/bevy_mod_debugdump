use bevy_ecs::{
    component::ComponentId,
    schedule::{NodeId, Schedule, ScheduleLabel, Schedules},
    world::World,
};
use bevy_utils::hashbrown::{HashMap, HashSet};

use crate::dot::DotGraph;

pub struct EventGraphContext {
    events_tracked: HashSet<ComponentId>,
    event_readers: HashMap<ComponentId, Vec<NodeId>>,
    event_writers: HashMap<ComponentId, Vec<NodeId>>,
    schedule: Box<dyn ScheduleLabel>,
}

/// Formats the events into a dot graph.
pub fn events_graph_dot(schedule: &Schedule, world: &World) -> EventGraphContext {
    let graph = schedule.graph();

    let mut events_tracked = HashSet::new();
    let mut event_readers = HashMap::<ComponentId, Vec<NodeId>>::new();
    let mut event_writers = HashMap::<ComponentId, Vec<NodeId>>::new();
    for (system_id, system, _condition) in graph.systems() {
        // TODO: make this configurable
        let system_name = graph.system_at(system_id).name();
        if system_name.starts_with("bevy_ecs::event::event_update_system<") {
            continue;
        }
        let accesses = system.component_access();
        for access in accesses.reads() {
            let component = world.components().get_info(access).unwrap();
            let name = component.name();
            if name.starts_with("bevy_ecs::event::Events") {
                events_tracked.insert(access);
                match event_readers.entry(access) {
                    bevy_utils::hashbrown::hash_map::Entry::Occupied(mut entry) => {
                        entry.get_mut().push(system_id)
                    }
                    bevy_utils::hashbrown::hash_map::Entry::Vacant(vacant) => {
                        vacant.insert([system_id].into());
                    }
                }
            }
        }
        for access in accesses.writes() {
            let component = world.components().get_info(access).unwrap();
            let name = component.name();
            if name.starts_with("bevy_ecs::event::Events") {
                events_tracked.insert(access);
                match event_writers.entry(access) {
                    bevy_utils::hashbrown::hash_map::Entry::Occupied(mut entry) => {
                        entry.get_mut().push(system_id)
                    }
                    bevy_utils::hashbrown::hash_map::Entry::Vacant(vacant) => {
                        vacant.insert([system_id].into());
                    }
                }
            }
        }
    }
    EventGraphContext {
        events_tracked,
        event_readers,
        event_writers,
        schedule: Box::new(schedule.label()),
    }
}

pub fn print_only_context(
    schedules: &bevy_ecs::schedule::Schedules,
    dot: &mut DotGraph,
    ctx: &EventGraphContext,
    world: &World,
    _settings: &crate::schedule_graph::Settings,
) {
    let schedule = schedules
        .iter()
        .find(|s| (*ctx.schedule).as_dyn_eq().dyn_eq(s.0.as_dyn_eq()))
        .unwrap()
        .1;
    let graph = schedule.graph();
    let all_writers = ctx
        .event_writers
        .values()
        .flatten()
        .copied()
        .collect::<HashSet<_>>();
    let all_readers = ctx
        .event_readers
        .values()
        .flatten()
        .copied()
        .collect::<HashSet<_>>();

    let all_systems = all_writers
        .iter()
        .chain(all_readers.iter())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<&NodeId>>();

    for s in all_systems {
        let name = &graph.get_system_at(*s).unwrap().name();
        let color = match (all_writers.contains(s), all_readers.contains(s)) {
            (true, false) => "yellow",
            (false, true) => "red",
            (true, true) => "orange",
            _ => panic!("Unexpected event handled."),
        };
        dot.add_node(
            name,
            &[
                ("color", color),
                ("label", &pretty_type_name::pretty_type_name_str(name)),
                ("tooltip", name),
                ("shape", "box"),
            ],
        );
    }

    for event in ctx.events_tracked.iter() {
        let readers = ctx.event_readers.get(event).cloned().unwrap_or_default();
        let writers = ctx.event_writers.get(event).cloned().unwrap_or_default();

        let component = world.components().get_info(*event).unwrap();

        // Relevant name is only what's inside "bevy::ecs::Events<(...)>"
        let full_name = component.name();
        let name = full_name.split_once('<').unwrap().1;
        let name = &name[0..name.len() - 1];
        let event_id = format!("event_{0}", event.index());
        dot.add_node(
            &event_id,
            &[
                ("color", "green"),
                ("label", &pretty_type_name::pretty_type_name_str(name)),
                ("tooltip", name),
                ("shape", "ellipse"),
            ],
        );
        for writer in writers {
            // We have to use full names, because nodeId is schedule specific, and I want to support multiple schedules displayed
            let system_name = graph.get_system_at(writer).unwrap().name();
            dot.add_edge(&system_name, &event_id, &[]);
        }
        for reader in readers {
            let system_name = graph.get_system_at(reader).unwrap().name();
            dot.add_edge(&event_id, &system_name, &[]);
        }
    }
}

pub fn print_context(
    schedules: &Schedules,
    ctxs: &Vec<EventGraphContext>,
    world: &World,
    settings: &crate::schedule_graph::Settings,
) -> String {
    let mut dot = DotGraph::new(
        "",
        "digraph",
        &[
            ("compound", "true"), // enable ltail/lhead
            ("splines", settings.style.edge_style.as_dot()),
            ("rankdir", settings.style.schedule_rankdir.as_dot()),
            ("bgcolor", &settings.style.color_background),
            ("fontname", &settings.style.fontname),
            ("nodesep", "0.15"),
        ],
    )
    .edge_attributes(&[("penwidth", &format!("{}", settings.style.penwidth_edge))])
    .node_attributes(&[("shape", "box"), ("style", "filled")]);

    for ctx in ctxs {
        print_only_context(schedules, &mut dot, ctx, world, settings);
    }
    dot.finish().to_string()
}
