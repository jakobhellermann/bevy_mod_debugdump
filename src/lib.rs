mod dot;

pub mod settings;

pub use settings::Settings;

use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Write,
};

use bevy_ecs::{
    schedule_v3::{NodeId, Schedule, ScheduleGraph, SystemSet},
    system::System,
    world::World,
};
use dot::DotGraph;
use petgraph::{prelude::DiGraphMap, Direction};

pub fn schedule_to_dot(schedule: &Schedule, world: &World, settings: &Settings) -> String {
    let graph = schedule.graph();
    let hierarchy = graph.hierarchy().graph();

    let mut dot = DotGraph::new(
        "schedule",
        "digraph",
        &[
            ("compound", "true"), // enable ltail/lhead
            ("splines", settings.style.edge_style.as_dot()),
            ("rankdir", settings.style.schedule_rankdir.as_dot()),
            ("bgcolor", &settings.style.color_background),
            ("fontname", &settings.style.fontname),
        ],
    )
    .node_attributes(&[
        ("shape", "box"),
        ("style", "filled"),
        ("fillcolor", &settings.style.color_system),
        ("color", &settings.style.color_system_border),
    ])
    .edge_attributes(&[("color", &settings.style.color_edge)]);

    let included_systems_sets = included_systems_sets(graph, settings);

    let mut system_sets: Vec<_> = graph.system_sets().collect();
    system_sets.sort_by_key(|&(node_id, ..)| node_id);

    // collect sets and systems
    let mut systems_freestanding = Vec::new();
    let mut systems_in_single_set = HashMap::<NodeId, Vec<_>>::new();
    let mut systems_in_multiple_sets = bevy_utils::HashMap::<Option<NodeId>, Vec<_>>::new();

    for (system_id, system, _base_set, _conditions) in graph
        .systems()
        .filter(|(id, ..)| included_systems_sets.contains(id))
    {
        let single_parent = iter_single(hierarchy_parents(system_id, graph));

        match single_parent {
            IterSingleResult::Empty => systems_freestanding.push((system_id, system)),
            IterSingleResult::Single(parent) => {
                systems_in_single_set
                    .entry(parent)
                    .or_default()
                    .push((system_id, system));
            }
            IterSingleResult::Multiple(parents) => {
                let first_common_ancestor = lowest_common_ancestor(&parents, hierarchy);

                systems_in_multiple_sets
                    .entry(first_common_ancestor)
                    .or_default()
                    .push((system_id, system))
            }
        }
    }

    let mut sets_freestanding = Vec::new();
    let mut sets_in_single_set = HashMap::<NodeId, Vec<_>>::new();
    let mut sets_in_multiple_sets = bevy_utils::HashMap::<Option<NodeId>, Vec<_>>::new();

    for &(set_id, set, _base_set_membership, _conditions) in system_sets
        .iter()
        .filter(|&&(id, ..)| graph.set_at(id).system_type().is_none())
        .filter(|(id, ..)| included_systems_sets.contains(id))
    {
        let single_parent = iter_single(hierarchy_parents(set_id, graph));

        match single_parent {
            IterSingleResult::Empty => sets_freestanding.push((set_id, set)),
            IterSingleResult::Single(parent) => {
                sets_in_single_set
                    .entry(parent)
                    .or_default()
                    .push((set_id, set));
            }
            IterSingleResult::Multiple(parents) => {
                let first_common_ancestor = lowest_common_ancestor(&parents, hierarchy);

                sets_in_multiple_sets
                    .entry(first_common_ancestor)
                    .or_default()
                    .push((set_id, set));
            }
        }
    }

    fn add_system_in_multiple_sets(
        dot: &mut DotGraph,
        system_id: NodeId,
        system: &(dyn System<In = (), Out = ()>),
        graph: &ScheduleGraph,
        included_systems_sets: &HashSet<NodeId>,
        settings: &Settings,
    ) {
        assert!(included_systems_sets.contains(&system_id));
        let mut name = system_name(system, settings);
        name.push_str("\nIn multiple sets");

        for parent in hierarchy_parents(system_id, graph) {
            assert!(included_systems_sets.contains(&parent));
            let parent_set = graph.set_at(parent);
            let _ = write!(&mut name, ", {parent_set:?}");

            dot.add_edge(
                &node_id(system_id, graph),
                &node_id(parent, graph),
                &[
                    ("dir", "none"),
                    ("color", &settings.style.multiple_set_edge_color),
                    ("lhead", &set_cluster_name(parent)),
                ],
            );
        }
        dot.add_node(&node_id(system_id, graph), &[("label", &name)]);
    }

    fn add_set_in_multiple_sets(
        dot: &mut DotGraph,
        set_id: NodeId,
        set: &dyn SystemSet,
        graph: &ScheduleGraph,
        settings: &Settings,
        sets_in_single_set: &HashMap<NodeId, Vec<(NodeId, &dyn SystemSet)>>,
        sets_in_multiple_sets: &bevy_utils::hashbrown::HashMap<
            Option<NodeId>,
            Vec<(NodeId, &dyn SystemSet)>,
        >,
        systems_in_single_set: &HashMap<NodeId, Vec<(NodeId, &(dyn System<In = (), Out = ()>))>>,
        systems_in_multiple_sets: &bevy_utils::hashbrown::HashMap<
            Option<NodeId>,
            Vec<(NodeId, &(dyn System<In = (), Out = ()>))>,
        >,
        included_systems_sets: &HashSet<NodeId>,
    ) {
        assert!(included_systems_sets.contains(&set_id));
        add_set(
            set_id,
            set,
            dot,
            graph,
            settings,
            &sets_in_single_set,
            &sets_in_multiple_sets,
            &systems_in_single_set,
            &systems_in_multiple_sets,
            &included_systems_sets,
        );

        for parent in hierarchy_parents(set_id, graph) {
            assert!(included_systems_sets.contains(&parent));
            dot.add_edge(
                &node_id(parent, graph),
                &node_id(set_id, graph),
                &[
                    ("dir", "none"),
                    ("color", &settings.style.multiple_set_edge_color),
                    ("ltail", &lref(parent, graph)),
                    ("lhead", &lref(set_id, graph)),
                ],
            );
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
        sets_in_multiple_sets: &bevy_utils::hashbrown::HashMap<
            Option<NodeId>,
            Vec<(NodeId, &dyn SystemSet)>,
        >,
        systems_in_single_set: &HashMap<NodeId, Vec<(NodeId, &(dyn System<In = (), Out = ()>))>>,
        systems_in_multiple_sets: &bevy_utils::hashbrown::HashMap<
            Option<NodeId>,
            Vec<(NodeId, &(dyn System<In = (), Out = ()>))>,
        >,
        included_systems_sets: &HashSet<NodeId>,
    ) {
        let name = format!("{set:?}");

        let system_set_cluster_name = node_index_name(set_id); // in sync with system_cluster_name
        let mut system_set_graph = DotGraph::subgraph(
            &system_set_cluster_name,
            &[
                ("label", &name),
                ("bgcolor", &settings.style.color_set),
                // ("color", &settings.style.color_set_border),
            ],
        );

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
                sets_in_multiple_sets,
                systems_in_single_set,
                systems_in_multiple_sets,
                included_systems_sets,
            );
        }

        for &(nested_set_id, nested_set) in sets_in_multiple_sets
            .get(&Some(set_id))
            .map(|vec| vec.as_slice())
            .unwrap_or_default()
        {
            add_set_in_multiple_sets(
                &mut system_set_graph,
                nested_set_id,
                nested_set,
                graph,
                settings,
                &sets_in_single_set,
                &sets_in_multiple_sets,
                &systems_in_single_set,
                &systems_in_multiple_sets,
                &included_systems_sets,
            );
        }

        let systems = systems_in_single_set
            .get(&set_id)
            .map(|systems| systems.as_slice())
            .unwrap_or(&[]);
        let show_systems = settings.include_single_system_in_set || systems.len() > 1;
        for &(system_id, system) in systems.iter() {
            let name = system_name(system, settings);
            if show_systems {
                system_set_graph.add_node(&node_id(system_id, graph), &[("label", name.as_str())]);
            } else {
                system_set_graph.add_node(
                    &node_id(system_id, graph),
                    &[("label", ""), ("style", "invis")],
                );
            }
        }

        for &(system_id, system) in systems_in_multiple_sets
            .get(&Some(set_id))
            .map(|vec| vec.as_slice())
            .unwrap_or_default()
        {
            add_system_in_multiple_sets(
                &mut system_set_graph,
                system_id,
                system,
                graph,
                &included_systems_sets,
                settings,
            );
        }

        dot.add_sub_graph(system_set_graph);
    }

    for &(set_id, set) in sets_freestanding.iter() {
        assert!(included_systems_sets.contains(&set_id));
        add_set(
            set_id,
            set,
            &mut dot,
            graph,
            settings,
            &sets_in_single_set,
            &sets_in_multiple_sets,
            &systems_in_single_set,
            &systems_in_multiple_sets,
            &included_systems_sets,
        );
    }
    for &(set_id, set) in sets_in_multiple_sets
        .get(&None)
        .map(|vec| vec.as_slice())
        .unwrap_or_default()
    {
        add_set_in_multiple_sets(
            &mut dot,
            set_id,
            set,
            graph,
            settings,
            &sets_in_single_set,
            &sets_in_multiple_sets,
            &systems_in_single_set,
            &systems_in_multiple_sets,
            &included_systems_sets,
        );
    }

    for &(system_id, system) in systems_freestanding.iter() {
        assert!(included_systems_sets.contains(&system_id));
        let name = system_name(system, settings);
        dot.add_node(&node_id(system_id, graph), &[("label", &name)]);
    }

    for &(system_id, system) in systems_in_multiple_sets
        .get(&None)
        .map(|vec| vec.as_slice())
        .unwrap_or_default()
    {
        add_system_in_multiple_sets(
            &mut dot,
            system_id,
            system,
            graph,
            &included_systems_sets,
            settings,
        );
    }

    let dependency = graph.dependency();
    for (from, to, ()) in dependency.graph().all_edges() {
        if !included_systems_sets.contains(&from) || !included_systems_sets.contains(&to) {
            continue;
        }

        dot.add_edge(
            &node_id(from, graph),
            &node_id(to, graph),
            &[("lhead", &lref(to, graph)), ("ltail", &lref(from, graph))],
        );
    }

    if settings.ambiguity_enable {
        let mut conflicting_systems = graph.conflicting_systems.to_vec();
        conflicting_systems.sort();

        for (system_a, system_b, conflicts) in conflicting_systems {
            if !included_systems_sets.contains(&system_a)
                || !included_systems_sets.contains(&system_b)
            {
                continue;
            }

            if conflicts.is_empty() && !settings.ambiguity_enable_on_world {
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
                        settings.style.ambiguity_bgcolor,
                        dot::html_escape(&pretty_name)
                    )
                });
                let trs = component_names.collect::<String>();
                format!(r#"<<table border="0" cellborder="0">{trs}</table>>"#)
            };
            let name_a = system_name(graph.system_at(system_a), settings);
            let name_b = system_name(graph.system_at(system_b), settings);

            dot.add_edge(
                &node_id(system_a, graph),
                &node_id(system_b, graph),
                &[
                    ("dir", "none"),
                    ("constraint", "false"),
                    ("color", &settings.style.ambiguity_color),
                    ("fontcolor", &settings.style.ambiguity_color),
                    ("label", &label),
                    ("labeltooltip", &format!("{name_a} -- {name_b}")),
                ],
            );
        }
    }

    dot.finish()
}

fn lowest_common_ancestor(
    parents: &[NodeId],
    hierarchy: &DiGraphMap<NodeId, ()>,
) -> Option<NodeId> {
    let parent = parents.last().unwrap();
    let mut common_ancestors: Vec<_> = ancestors_of_node(*parent, hierarchy).collect();

    for &other_parent in parents[0..parents.len() - 1].iter().rev() {
        common_ancestors.retain(|&ancestor| {
            ancestors_of_node(other_parent, hierarchy)
                .any(|other_ancestor| other_ancestor == ancestor)
        })
    }

    let first_common_ancestor = common_ancestors.first().copied();
    first_common_ancestor
}

fn included_systems_sets(graph: &ScheduleGraph, settings: &Settings) -> HashSet<NodeId> {
    let hierarchy = graph.hierarchy().graph();

    let root_sets = hierarchy.nodes().filter(|&node| {
        node.is_set()
            && graph.set_at(node).system_type().is_none()
            && hierarchy
                .neighbors_directed(node, Direction::Incoming)
                .next()
                .is_none()
    });

    let systems_of_interest: HashSet<NodeId> = graph
        .systems()
        .filter(|&(_, system, _, _)| settings.include_system(system))
        .map(|(id, ..)| id)
        .collect();

    fn include_ancestors(
        id: NodeId,
        hierarchy: &DiGraphMap<NodeId, ()>,
        included_systems_sets: &mut HashSet<NodeId>,
    ) {
        let parents = hierarchy.neighbors_directed(id, Direction::Incoming);

        for parent in parents {
            included_systems_sets.insert(parent);
            include_ancestors(parent, hierarchy, included_systems_sets);
        }
    }

    let mut included_systems_sets = systems_of_interest.clone();
    included_systems_sets.extend(root_sets);

    for &id in &systems_of_interest {
        include_ancestors(id, hierarchy, &mut included_systems_sets);
    }

    if settings.ambiguity_enable {
        for &(a, b, ref conflicts) in graph.conflicting_systems() {
            if !systems_of_interest.contains(&a) || !systems_of_interest.contains(&b) {
                continue;
            }

            if !settings.ambiguity_enable_on_world && conflicts.is_empty() {
                continue;
            }

            included_systems_sets.insert(a);
            included_systems_sets.insert(b);
        }
    }

    for (from, to, ()) in graph.dependency().graph().all_edges() {
        if systems_of_interest.contains(&from) {
            included_systems_sets.insert(to);
            include_ancestors(to, hierarchy, &mut included_systems_sets);
        }

        if systems_of_interest.contains(&to) {
            included_systems_sets.insert(from);
            include_ancestors(to, hierarchy, &mut included_systems_sets);
        }
    }

    included_systems_sets
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

fn system_name(system: &dyn System<In = (), Out = ()>, settings: &Settings) -> String {
    let name = system.name();
    if settings.prettify_system_names {
        pretty_type_name::pretty_type_name_str(&name)
    } else {
        name.into()
    }
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
                    .find_map(|(node_id, system, _, _)| {
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
    Multiple(Vec<T>),
}
fn iter_single<T>(mut iter: impl Iterator<Item = T>) -> IterSingleResult<T> {
    let Some(first) = iter.next() else { return IterSingleResult::Empty };
    match iter.next() {
        Some(second) => {
            let mut items = Vec::with_capacity(iter.size_hint().0 + 2);
            items.push(first);
            items.push(second);
            items.extend(iter);
            IterSingleResult::Multiple(items)
        }
        None => IterSingleResult::Single(first),
    }
}

fn ancestors_of_node(
    node_id: NodeId,
    graph: &DiGraphMap<NodeId, ()>,
) -> impl Iterator<Item = NodeId> + '_ {
    let mut queue = VecDeque::with_capacity(1);
    queue.push_back(node_id);
    Ancestors { queue, graph }
}

struct Ancestors<'a> {
    queue: VecDeque<NodeId>,
    graph: &'a DiGraphMap<NodeId, ()>,
}

impl Iterator for Ancestors<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.queue.pop_front()?;

        self.queue
            .extend(self.graph.neighbors_directed(item, Direction::Incoming));

        Some(item)
    }
}

fn hierarchy_parents<'a>(
    node: NodeId,
    graph: &'a ScheduleGraph,
) -> impl Iterator<Item = NodeId> + 'a {
    let hierarchy = graph.hierarchy().graph();
    hierarchy
        .neighbors_directed(node, Direction::Incoming)
        .filter(|&parent| graph.set_at(parent).system_type().is_none())
}
