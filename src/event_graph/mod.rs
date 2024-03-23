use bevy_ecs::{
    schedule::{NodeId, Schedule},
    world::World,
};
use bevy_utils::hashbrown::{HashMap, HashSet};

use crate::dot::DotGraph;

/// Formats the events into a dot graph.
pub fn events_graph_dot(
    schedule: &Schedule,
    world: &World,
    settings: &crate::schedule_graph::Settings,
) -> String {
    let graph = schedule.graph();

    /*
    let hierarchy = graph.hierarchy().graph();
    let dependency = graph.dependency().graph();
    */

    let mut events_tracked = HashSet::new();
    let mut event_readers = HashMap::<&str, Vec<NodeId>>::new();
    let mut event_writers = HashMap::<&str, Vec<NodeId>>::new();
    for (system_id, system, _condition) in graph.systems() {
        let accesses = system.component_access();
        for access in accesses.reads() {
            let component = world.components().get_info(access).unwrap();
            let name = component.name();
            if name.starts_with("bevy_ecs::event::Events") {
                // TODO: avoid relying on name, use TypeId ?
                events_tracked.insert(name);
                match event_readers.entry(name) {
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
            // TODO: avoid copying code?
            let component = world.components().get_info(access).unwrap();
            let name = component.name();
            if name.starts_with("bevy_ecs::event::Events") {
                events_tracked.insert(name);

                match event_writers.entry(name) {
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

    let all_writers = event_writers
        .values()
        .flatten()
        .copied()
        .collect::<HashSet<_>>();
    let all_readers = event_readers
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
            &node_string(s),
            &[
                ("color", color),
                (
                    "label",
                    name.split('<').next().unwrap().split("::").last().unwrap(),
                ),
                ("tooltip", name),
                ("shape", "box"),
            ],
        );
    }

    for event in events_tracked {
        let readers = event_readers.entry(event).or_default();
        let writers = event_writers.entry(event).or_default();

        let name = event
            .split_once('<')
            .unwrap()
            .1 /* .split("::").last()
            .unwrap()*/;
        let name = &name[0..name.len() - 1];
        dot.add_node(
            event,
            &[
                ("color", "green"),
                ("label", name),
                ("tooltip", "nice event"),
                ("shape", "ellipse"),
            ],
        );
        for writer in writers {
            dot.add_edge(&node_string(writer), event, &[]);
        }
        for reader in readers {
            dot.add_edge(event, &node_string(reader), &[]);
        }
    }
    dot.finish().to_string()
}

/// Internal but we use that as identifiers
fn node_index(node_id: &NodeId) -> usize {
    match node_id {
        NodeId::System(index) | NodeId::Set(index) => *index,
    }
}
/// Internal but we use that as identifiers
fn node_string(node_id: &NodeId) -> String {
    node_index(node_id).to_string()
}
