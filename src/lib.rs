mod dot;

use std::collections::HashMap;

use bevy_ecs::{
    scheduling::{NodeId, Schedule, ScheduleLabel},
    system::System,
};
use dot::DotGraph;
use petgraph::Direction;

const SCHEDULE_RANKDIR: &str = "TD";
const MULTIPLE_SET_EDGE_COLOR: &str = "red";

pub fn schedule_to_dot(schedule_label: &dyn ScheduleLabel, schedule: &Schedule) -> String {
    let name = format!("{:?}", schedule_label);
    let graph = schedule.graph();

    let mut dot = DotGraph::new(
        &name,
        "digraph",
        &[
            ("compound", "true"), // enable ltail/lhead
            ("rankdir", SCHEDULE_RANKDIR),
        ],
    );

    let hierarchy = &graph.hierarchy().graph;

    let hierarchy_parents = |node| {
        hierarchy
            .neighbors_directed(node, Direction::Incoming)
            .filter(|&parent| !graph.set_at(parent).is_system_type())
    };

    let mut system_sets: Vec<_> = graph.system_sets().collect();
    system_sets.sort_by_key(|&(node_id, ..)| node_id);

    let mut systems_freestanding = Vec::new();
    let mut systems_in_single_set = HashMap::<NodeId, Vec<_>>::new();
    let mut systems_in_multiple_sets = Vec::new();

    for (system_id, system, _conditions) in graph.systems() {
        let single_parent = iter_single(hierarchy_parents(system_id));

        match single_parent {
            IterSingleResult::Empty => systems_freestanding.push((system_id, system)),
            IterSingleResult::Single(parent) => {
                systems_in_single_set
                    .entry(parent)
                    .or_default()
                    .push((system_id, system));
            }
            IterSingleResult::Multiple => systems_in_multiple_sets.push((system_id, system)),
        }
    }

    for &(set_id, set, _conditions) in system_sets.iter() {
        let name = format!("{set:?}");
        if set.is_system_type() {
            continue;
        };

        let system_set_cluster_name = node_index_name(set_id); // in sync with system_cluster_name
        let mut system_set_graph =
            DotGraph::subgraph(&system_set_cluster_name, &[("label", &name)]);

        system_set_graph.add_invisible_node(&marker_name(set_id));

        for &(system_id, system) in systems_in_single_set
            .get(&set_id)
            .map(|systems| systems.as_slice())
            .unwrap_or(&[])
        {
            let name = system_name(system);
            system_set_graph.add_node(&node_id(system_id), &[("label", &name)]);
        }

        dot.add_sub_graph(system_set_graph);
    }

    for &(system_id, system) in systems_freestanding.iter() {
        let name = system_name(system);
        dot.add_node(&node_id(system_id), &[("label", &name)]);
    }

    for &(system_id, system) in systems_in_multiple_sets.iter() {
        let name = system_name(system);
        dot.add_node(&node_id(system_id), &[("label", &name)]);

        for parent in hierarchy_parents(system_id) {
            dot.add_edge(
                &node_id(parent),
                &node_id(system_id),
                &[
                    ("dir", "none"),
                    ("color", MULTIPLE_SET_EDGE_COLOR),
                    ("ltail", &set_cluster_name(parent)),
                ],
            );
        }
    }

    let dependency = graph.dependency();
    for (from, to, ()) in dependency.graph.all_edges() {
        let is_non_system_set = |id: NodeId| id.is_set() && !graph.set_at(id).is_system_type();

        let ltail = is_non_system_set(from)
            .then(|| set_cluster_name(from))
            .unwrap_or_default();
        let lhead = is_non_system_set(to)
            .then(|| set_cluster_name(to))
            .unwrap_or_default();

        dot.add_edge(
            &node_id(from),
            &node_id(to),
            &[("lhead", &lhead), ("ltail", &ltail)],
        );
    }

    dot.finish()
}

fn set_cluster_name(id: NodeId) -> String {
    assert!(id.is_set());
    format!("cluster{}", node_index_name(id))
}

fn system_name(system: &dyn System<In = (), Out = ()>) -> String {
    let name = system.name();
    pretty_type_name::pretty_type_name_str(&name)
}

fn node_index_name(node_id: NodeId) -> String {
    format!("node_{:?}", node_id)
}
fn marker_name(node_id: NodeId) -> String {
    assert!(node_id.is_set());
    format!("set_marker_node_{:?}", node_id)
}

fn node_id(node_id: NodeId) -> String {
    match node_id {
        NodeId::System(_) => node_index_name(node_id),
        NodeId::Set(_) => marker_name(node_id),
    }
}

enum IterSingleResult<T> {
    Empty,
    Single(T),
    Multiple,
}
fn iter_single<T>(mut iter: impl Iterator<Item = T>) -> IterSingleResult<T> {
    let Some(first) = iter.next() else { return IterSingleResult::Empty };
    let None = iter.next() else { return IterSingleResult::Multiple };
    IterSingleResult::Single(first)
}
