mod dot;

use bevy_ecs::{
    schedule::{NodeId, Schedule, ScheduleLabel},
    system::System,
};
use dot::DotGraph;

const SCHEDULE_RANKDIR: &str = "TD";

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

    eprintln!("\n");

    let mut system_sets: Vec<_> = graph.system_sets().collect();
    system_sets.sort_by_key(|&(node_id, ..)| node_id);

    let systems: Vec<_> = graph.systems().collect();

    for &(set_id, set, _conditions) in system_sets.iter() {
        let name = format!("{set:?}");
        if name.starts_with("SystemTypeSet") {
            continue; // skip for now
        }

        let system_set_cluster_name = node_index_name(set_id); // in sync with system_cluster_name
        let mut system_set_graph =
            DotGraph::subgraph(&system_set_cluster_name, &[("label", &name)]);

        system_set_graph.add_invisible_node(&marker_name(set_id));

        dot.add_sub_graph(system_set_graph);
    }

    for &(system_id, system, _conditions) in systems.iter() {
        let name = system_name(system);
        dot.add_node(&node_id(system_id), &[("label", &name)]);
    }

    let dependency = graph.dependency();
    for (from, to, ()) in dependency.graph.all_edges() {
        let ltail = from
            .is_set()
            .then(|| set_cluster_name(from))
            .unwrap_or_default();
        let lhead = to
            .is_set()
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
