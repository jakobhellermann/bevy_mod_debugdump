mod dot;

use std::{collections::HashMap, fmt::Write};

use bevy_ecs::{
    scheduling::{NodeId, Schedule, ScheduleGraph, ScheduleLabel, SystemSet},
    system::System,
};
use dot::DotGraph;
use petgraph::Direction;

const SCHEDULE_RANKDIR: &str = "LR";
const MULTIPLE_SET_EDGE_COLOR: &str = "red";

pub struct Settings {
    pub show_single_system_in_set: bool,
    pub include_system: Box<dyn Fn(&dyn System<In = (), Out = ()>) -> bool>,
}
impl Settings {
    fn include_system(&self, system: &dyn System<In = (), Out = ()>) -> bool {
        (self.include_system)(system)
    }
}

pub fn schedule_to_dot(
    schedule_label: &dyn ScheduleLabel,
    schedule: &Schedule,
    settings: &Settings,
) -> String {
    let name = format!("{:?}", schedule_label);
    let graph = schedule.graph();

    let mut dot = DotGraph::new(
        &name,
        "digraph",
        &[
            ("compound", "true"), // enable ltail/lhead
            ("rankdir", SCHEDULE_RANKDIR),
        ],
    )
    .node_attributes(&[("shape", "box")]);

    let hierarchy = &graph.hierarchy().graph;

    // utilities
    let hierarchy_parents = |node| {
        hierarchy
            .neighbors_directed(node, Direction::Incoming)
            .filter(|&parent| graph.set_at(parent).system_type().is_none())
    };

    let mut system_sets: Vec<_> = graph.system_sets().collect();
    system_sets.sort_by_key(|&(node_id, ..)| node_id);

    // collect sets and systems
    let mut systems_freestanding = Vec::new();
    let mut systems_in_single_set = HashMap::<NodeId, Vec<_>>::new();
    let mut systems_in_multiple_sets = Vec::new();

    for (system_id, system, _conditions) in graph.systems() {
        if !settings.include_system(system) {
            continue;
        }

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

    let mut sets_freestanding = Vec::new();
    let mut sets_in_single_set = HashMap::<NodeId, Vec<_>>::new();
    let mut sets_in_multiple_sets = Vec::new();

    for &(set_id, set, _conditions) in system_sets
        .iter()
        .filter(|&&(id, ..)| graph.set_at(id).system_type().is_none())
    {
        let single_parent = iter_single(hierarchy_parents(set_id));

        match single_parent {
            IterSingleResult::Empty => sets_freestanding.push((set_id, set)),
            IterSingleResult::Single(parent) => {
                sets_in_single_set
                    .entry(parent)
                    .or_default()
                    .push((set_id, set));
            }
            IterSingleResult::Multiple => sets_in_multiple_sets.push((set_id, set)),
        }
    }

    // add regular set and system hierarchy
    fn add_set(
        set_id: NodeId,
        set: &dyn SystemSet,
        dot: &mut DotGraph,
        graph: &ScheduleGraph,
        settings: &Settings,
        sets_in_single_set: &HashMap<NodeId, Vec<(NodeId, &dyn SystemSet)>>,
        systems_in_single_set: &HashMap<NodeId, Vec<(NodeId, &(dyn System<In = (), Out = ()>))>>,
    ) {
        let name = format!("{set:?}");

        let system_set_cluster_name = node_index_name(set_id); // in sync with system_cluster_name
        let mut system_set_graph =
            DotGraph::subgraph(&system_set_cluster_name, &[("label", &name)]);

        system_set_graph.add_invisible_node(&marker_name(set_id));

        for &(nested_set_id, nested_set) in sets_in_single_set
            .get(&set_id)
            .map(|sets| sets.as_slice())
            .unwrap_or(&[])
        {
            add_set(
                nested_set_id,
                nested_set,
                &mut system_set_graph,
                graph,
                settings,
                sets_in_single_set,
                systems_in_single_set,
            );
        }

        let systems = systems_in_single_set
            .get(&set_id)
            .map(|systems| systems.as_slice())
            .unwrap_or(&[]);
        let show_systems = settings.show_single_system_in_set || systems.len() > 1;
        for &(system_id, system) in systems {
            if !settings.include_system(system) {
                continue;
            }

            let name = system_name(system);
            if show_systems {
                system_set_graph.add_node(&node_id(system_id, graph), &[("label", name.as_str())]);
            } else {
                system_set_graph.add_node(
                    &node_id(system_id, graph),
                    &[("label", ""), ("style", "invis")],
                );
            }
        }

        dot.add_sub_graph(system_set_graph);
    }

    for &(set_id, set) in sets_freestanding.iter() {
        add_set(
            set_id,
            set,
            &mut dot,
            graph,
            settings,
            &sets_in_single_set,
            &systems_in_single_set,
        );
    }
    for &(set_id, set) in sets_in_multiple_sets.iter() {
        // TODO
        add_set(
            set_id,
            set,
            &mut dot,
            graph,
            settings,
            &sets_in_single_set,
            &systems_in_single_set,
        );

        for parent in hierarchy_parents(set_id) {
            dot.add_edge(
                &node_id(parent, graph),
                &node_id(set_id, graph),
                &[
                    ("dir", "none"),
                    ("color", MULTIPLE_SET_EDGE_COLOR),
                    ("ltail", &set_cluster_name(parent)),
                    ("lhead", &set_cluster_name(set_id)),
                ],
            );
        }
    }

    for &(system_id, system) in systems_freestanding.iter() {
        let name = system_name(system);
        dot.add_node(&node_id(system_id, graph), &[("label", &name)]);
    }

    for &(system_id, system) in systems_in_multiple_sets.iter() {
        let mut name = system_name(system);
        name.push_str("\nIn multiple sets");

        for parent in hierarchy_parents(system_id) {
            let parent_set = graph.set_at(parent);
            let _ = write!(&mut name, ", {parent_set:?}");

            dot.add_edge(
                &node_id(parent, graph),
                &node_id(system_id, graph),
                &[
                    ("dir", "none"),
                    ("color", MULTIPLE_SET_EDGE_COLOR),
                    ("ltail", &set_cluster_name(parent)),
                ],
            );
        }

        dot.add_node(&node_id(system_id, graph), &[("label", &name)]);
    }

    let dependency = graph.dependency();
    for (from, to, ()) in dependency.graph.all_edges() {
        let is_non_system_set =
            |id: NodeId| id.is_set() && graph.set_at(id).system_type().is_none();

        let ltail = is_non_system_set(from)
            .then(|| set_cluster_name(from))
            .unwrap_or_default();
        let lhead = is_non_system_set(to)
            .then(|| set_cluster_name(to))
            .unwrap_or_default();

        dot.add_edge(
            &node_id(from, graph),
            &node_id(to, graph),
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

fn node_id(node_id: NodeId, graph: &ScheduleGraph) -> String {
    match node_id {
        NodeId::System(_) => node_index_name(node_id),
        NodeId::Set(_) => {
            let set = graph.set_at(node_id);
            if let Some(system_type) = set.system_type() {
                let system_node = graph
                    .systems()
                    .find_map(|(node_id, system, _)| {
                        (system.type_id() == system_type).then_some(node_id)
                    })
                    .unwrap();
                node_index_name(system_node)
            } else {
                marker_name(node_id)
            }
        }
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
