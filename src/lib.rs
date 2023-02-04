mod dot;

use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
};

use bevy_ecs::{
    schedule_v3::{NodeId, Schedule, ScheduleGraph, SystemSet},
    system::System,
    world::World,
};
use dot::DotGraph;
use petgraph::Direction;

pub enum RankDir {
    TopDown,
    LeftRight,
}
impl RankDir {
    fn as_dot(&self) -> &'static str {
        match self {
            RankDir::TopDown => "TD",
            RankDir::LeftRight => "LR",
        }
    }
}

#[derive(Clone, Copy)]
pub enum EdgeStyle {
    None,
    Line,
    Polyline,
    Curved,
    Ortho,
    Spline,
}
impl EdgeStyle {
    pub fn as_dot(&self) -> &'static str {
        match self {
            EdgeStyle::None => "none",
            EdgeStyle::Line => "line",
            EdgeStyle::Polyline => "polyline",
            EdgeStyle::Curved => "curved",
            EdgeStyle::Ortho => "ortho",
            EdgeStyle::Spline => "spline",
        }
    }
}

const MULTIPLE_SET_EDGE_COLOR: &str = "red";

pub struct Settings {
    pub schedule_rankdir: RankDir,
    pub edge_style: EdgeStyle,

    pub show_single_system_in_set: bool,
    pub include_system: Box<dyn Fn(&dyn System<In = (), Out = ()>) -> bool>,

    pub show_ambiguities: bool,
    pub show_ambiguities_on_world: bool,
    pub ambiguity_color: String,
    pub ambiguity_bgcolor: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schedule_rankdir: RankDir::LeftRight,
            edge_style: EdgeStyle::Spline,

            show_single_system_in_set: true,
            include_system: Box::new(|_| true),
            show_ambiguities: true,
            show_ambiguities_on_world: false,
            ambiguity_color: "blue".into(),
            ambiguity_bgcolor: "#d3d3d3".into(),
        }
    }
}
impl Settings {
    fn include_system(&self, system: &dyn System<In = (), Out = ()>) -> bool {
        (self.include_system)(system)
    }
}

pub fn schedule_to_dot(schedule: &Schedule, world: &World, settings: &Settings) -> String {
    let graph = schedule.graph();
    let hierarchy = &graph.hierarchy().graph;

    let mut dot = DotGraph::new(
        "schedule",
        "digraph",
        &[
            ("splines", settings.edge_style.as_dot()),
            ("compound", "true"), // enable ltail/lhead
            ("rankdir", settings.schedule_rankdir.as_dot()),
        ],
    )
    .node_attributes(&[("shape", "box")]);

    // utilities
    let hierarchy_parents = |node| {
        hierarchy
            .neighbors_directed(node, Direction::Incoming)
            .filter(|&parent| graph.set_at(parent).system_type().is_none())
    };

    let included_systems_sets = included_systems_sets(graph, settings);

    let mut system_sets: Vec<_> = graph.system_sets().collect();
    system_sets.sort_by_key(|&(node_id, ..)| node_id);

    // collect sets and systems
    let mut systems_freestanding = Vec::new();
    let mut systems_in_single_set = HashMap::<NodeId, Vec<_>>::new();
    let mut systems_in_multiple_sets = Vec::new();

    for (system_id, system, _conditions) in graph.systems() {
        if !included_systems_sets.contains(&system_id) {
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
        included_systems_sets: &HashSet<NodeId>,
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
                included_systems_sets,
            );
        }

        let systems = systems_in_single_set
            .get(&set_id)
            .map(|systems| systems.as_slice())
            .unwrap_or(&[]);
        let show_systems = settings.show_single_system_in_set || systems.len() > 1;
        for &(system_id, system) in systems {
            if !included_systems_sets.contains(&system_id) {
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
            &included_systems_sets,
        );
    }
    for &(set_id, set) in sets_in_multiple_sets.iter() {
        add_set(
            set_id,
            set,
            &mut dot,
            graph,
            settings,
            &sets_in_single_set,
            &systems_in_single_set,
            &included_systems_sets,
        );

        for parent in hierarchy_parents(set_id) {
            dot.add_edge(
                &node_id(parent, graph),
                &node_id(set_id, graph),
                &[
                    ("dir", "none"),
                    ("color", MULTIPLE_SET_EDGE_COLOR),
                    ("ltail", &lref(parent, graph)),
                    ("lhead", &lref(set_id, graph)),
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
        if !included_systems_sets.contains(&from) && !included_systems_sets.contains(&to) {
            continue;
        }

        dot.add_edge(
            &node_id(from, graph),
            &node_id(to, graph),
            &[("lhead", &lref(to, graph)), ("ltail", &lref(from, graph))],
        );
    }

    if settings.show_ambiguities {
        let mut conflicting_systems = graph.conflicting_systems.to_vec();
        conflicting_systems.sort();

        for (system_a, system_b, conflicts) in conflicting_systems {
            if conflicts.is_empty() && !settings.show_ambiguities_on_world {
                continue;
            }

            let label = if conflicts.is_empty() {
                "World".to_owned()
            } else {
                let component_names = conflicts.iter().map(|&component_id| {
                    let component_name = world.components().get_info(component_id).unwrap().name();
                    let pretty_name = pretty_type_name::pretty_type_name_str(&component_name);

                    format!(
                        r#"<tr><td bgcolor="{}">{}</td></tr>"#,
                        settings.ambiguity_bgcolor,
                        dot::html_escape(&pretty_name)
                    )
                });
                let trs = component_names.collect::<String>();
                format!(r#"<<table border="0" cellborder="0">{trs}</table>>"#)
            };
            let name_a = system_name(graph.system_at(system_a));
            let name_b = system_name(graph.system_at(system_b));

            dot.add_edge(
                &node_id(system_a, graph),
                &node_id(system_b, graph),
                &[
                    ("dir", "none"),
                    ("constraint", "false"),
                    ("color", &settings.ambiguity_color),
                    ("fontcolor", &settings.ambiguity_color),
                    ("label", &label),
                    ("labeltooltip", &format!("{name_a} -- {name_b}")),
                ],
            );
        }
    }

    dot.finish()
}

fn included_systems_sets(graph: &ScheduleGraph, settings: &Settings) -> HashSet<NodeId> {
    let mut included_systems: HashSet<NodeId> = graph
        .systems()
        .filter(|&(.., system, _)| settings.include_system(system))
        .map(|(id, ..)| id)
        .collect();
    included_systems.extend(graph.system_sets().map(|(id, _, _)| id));

    if settings.show_ambiguities {
        for &(a, b, ref conflicts) in graph.conflicting_systems() {
            if !settings.show_ambiguities_on_world && conflicts.is_empty() {
                continue;
            }
            included_systems.insert(a);
            included_systems.insert(b);
        }
    }
    for (from, to, ()) in graph.dependency().graph.all_edges() {
        if included_systems.contains(&from) {
            included_systems.insert(to);
        }
        if included_systems.contains(&to) {
            included_systems.insert(from);
        }
    }

    included_systems
}

fn is_non_system_set(node_id: NodeId, graph: &ScheduleGraph) -> bool {
    node_id.is_set() && graph.set_at(node_id).system_type().is_none()
}

// lhead/ltail
fn lref(node_id: NodeId, graph: &ScheduleGraph) -> String {
    is_non_system_set(node_id, graph)
        .then(|| set_cluster_name(node_id))
        .unwrap_or_default()
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
