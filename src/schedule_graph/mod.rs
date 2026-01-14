pub mod settings;
pub mod system_style;

use bevy_platform::collections::hash_map::HashMap;
use bevy_platform::collections::hash_set::HashSet;
pub use settings::Settings;

use std::{any::TypeId, borrow::Cow, collections::VecDeque, fmt::Write, sync::atomic::AtomicUsize};

use crate::dot::DotGraph;
use bevy_ecs::{
    schedule::{
        graph::{DiGraph, Direction},
        ApplyDeferred, NodeId, Schedule, ScheduleGraph, SystemKey, SystemSet,
    },
    system::ScheduleSystem,
    world::World,
};

/// Formats the schedule into a dot graph.
pub fn schedule_graph_dot(schedule: &Schedule, world: &World, settings: &Settings) -> String {
    let graph = schedule.graph();
    let hierarchy = graph.hierarchy().graph();

    let mut dependency = graph.dependency().graph().clone();
    if settings.remove_transitive_edges {
        remove_transitive_edges(&mut dependency);
    }

    let included_systems_sets = included_systems_sets(graph, settings);

    let mut system_sets: Vec<_> = graph.system_sets.iter().collect();
    system_sets.sort_by_key(|&(node_id, ..)| node_id);

    // collect sets and systems
    let mut systems_freestanding = Vec::new();
    let mut systems_in_single_set = HashMap::<NodeId, Vec<_>>::default();
    let mut systems_in_multiple_sets = HashMap::<Option<NodeId>, Vec<_>>::default();

    for (system_id, system, _condition) in graph
        .systems
        .iter()
        .filter(|(id, ..)| included_systems_sets.contains(&NodeId::System(*id)))
    {
        let node_id = NodeId::System(system_id);
        let single_parent = iter_single(hierarchy_parents(node_id, graph));

        match single_parent {
            IterSingleResult::Empty => systems_freestanding.push((node_id, system)),
            IterSingleResult::Single(parent) => {
                systems_in_single_set
                    .entry(parent)
                    .or_default()
                    .push((node_id, system));
            }
            IterSingleResult::Multiple(parents) => {
                let first_common_ancestor = lowest_common_ancestor(&parents, hierarchy);

                systems_in_multiple_sets
                    .entry(first_common_ancestor)
                    .or_default()
                    .push((node_id, system))
            }
        }
    }

    let mut sets_freestanding = Vec::new();
    let mut sets_in_single_set = HashMap::<NodeId, Vec<_>>::default();
    let mut sets_in_multiple_sets = HashMap::<Option<NodeId>, Vec<_>>::default();

    let mut collapsed_sets = HashSet::default();
    let mut collapsed_set_children = HashMap::default();

    for &(set_id, set, _condition) in system_sets
        .iter()
        .filter(|&&(id, ..)| graph.system_sets.get(id).unwrap().system_type().is_none())
        .filter(|(id, ..)| included_systems_sets.contains(&NodeId::Set(*id)))
    {
        let node_id = NodeId::Set(set_id);
        let single_parent = iter_single(hierarchy_parents(node_id, graph));

        match single_parent {
            IterSingleResult::Empty => sets_freestanding.push((node_id, set)),
            IterSingleResult::Single(parent) => {
                sets_in_single_set
                    .entry(parent)
                    .or_default()
                    .push((node_id, set));
            }
            IterSingleResult::Multiple(parents) => {
                let first_common_ancestor = lowest_common_ancestor(&parents, hierarchy);

                sets_in_multiple_sets
                    .entry(first_common_ancestor)
                    .or_default()
                    .push((node_id, set));
            }
        }
    }

    for &(set_id, ..) in system_sets
        .iter()
        .filter(|&&(id, ..)| graph.system_sets.get(id).unwrap().system_type().is_none())
        .filter(|(id, ..)| included_systems_sets.contains(&NodeId::Set(*id)))
    {
        let node_id = NodeId::Set(set_id);
        if settings.collapse_single_system_sets {
            let children = systems_in_single_set
                .get(&node_id)
                .map_or(&[] as &[_], |vec| vec.as_slice());
            let children_in_multiple = systems_in_multiple_sets
                .get(&Some(node_id))
                .map_or(&[] as &[_], |vec| vec.as_slice());

            let children_sets_empty = sets_in_single_set
                .get(&node_id)
                .is_none_or(|vec| vec.is_empty());
            let children_sets_in_multiple_empty = systems_in_multiple_sets
                .get(&Some(node_id))
                .is_none_or(|vec| vec.is_empty());

            if children_in_multiple.is_empty()
                && children.len() <= 1
                && children_sets_empty
                && children_sets_in_multiple_empty
            {
                collapsed_sets.insert(node_id);

                for &(child, ..) in children {
                    collapsed_set_children.insert(child, node_id);
                }
                for &(child, ..) in children_in_multiple {
                    collapsed_set_children.insert(child, node_id);
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

    let context = ScheduleGraphContext {
        settings,
        world,
        graph: schedule.graph(),
        dependency: &mut dependency,
        included_systems_sets,
        systems_freestanding,
        systems_in_single_set,
        systems_in_multiple_sets,
        sets_freestanding,
        sets_in_single_set,
        sets_in_multiple_sets,
        collapsed_sets,
        collapsed_set_children,

        color_edge_idx: AtomicUsize::new(0),
    };

    context.add_sets(&mut dot);
    context.add_freestanding_systems(&mut dot);
    context.add_dependencies(&mut dot);

    if settings.ambiguity_enable {
        context.add_ambiguities(&mut dot);
    }

    dot.finish()
}

struct ScheduleGraphContext<'a> {
    settings: &'a Settings,
    world: &'a World,

    graph: &'a ScheduleGraph,
    dependency: &'a DiGraph<NodeId>,

    included_systems_sets: HashSet<NodeId>,

    systems_freestanding: Vec<(NodeId, &'a ScheduleSystem)>,
    systems_in_single_set: HashMap<NodeId, Vec<(NodeId, &'a ScheduleSystem)>>,
    systems_in_multiple_sets: HashMap<Option<NodeId>, Vec<(NodeId, &'a ScheduleSystem)>>,

    sets_freestanding: Vec<(NodeId, &'a dyn SystemSet)>,
    sets_in_single_set: HashMap<NodeId, Vec<(NodeId, &'a dyn SystemSet)>>,
    sets_in_multiple_sets: HashMap<Option<NodeId>, Vec<(NodeId, &'a dyn SystemSet)>>,

    collapsed_sets: HashSet<NodeId>,
    // map from child to collapsed set
    collapsed_set_children: HashMap<NodeId, NodeId>,

    color_edge_idx: AtomicUsize,
}

impl ScheduleGraphContext<'_> {
    /// Add sets with systems recursively, as well as sets belonging to multiple sets without a common ancestor
    fn add_sets(&self, dot: &mut DotGraph) {
        for &(set_id, set) in self.sets_freestanding.iter() {
            assert!(self.included_systems_sets.contains(&set_id));
            self.add_set(set_id, set, dot);
        }

        for &(set_id, set) in self
            .sets_in_multiple_sets
            .get(&None)
            .map(|vec| vec.as_slice())
            .unwrap_or_default()
        {
            self.add_set_in_multiple_sets(dot, set_id, set);
        }
    }

    /// Add freestanding systems that do not belong to a set, as well as systems in multiple sets without a common ancestor
    fn add_freestanding_systems(&self, dot: &mut DotGraph) {
        for &(system_id, system) in self.systems_freestanding.iter() {
            assert!(self.included_systems_sets.contains(&system_id));
            let name = self.system_name(system);
            dot.add_node(
                &node_index_name(system_id),
                &[("label", &name), ("tooltip", &system.name())],
            );
        }

        for &(system_id, system) in self
            .systems_in_multiple_sets
            .get(&None)
            .map(|vec| vec.as_slice())
            .unwrap_or_default()
        {
            self.add_system_in_multiple_sets(dot, system_id, system);
        }
    }

    /// Add dependency edges between nodes
    fn add_dependencies(&self, dot: &mut DotGraph) {
        for (from, to) in self.dependency.all_edges() {
            if !self.included_systems_sets.contains(&from)
                || !self.included_systems_sets.contains(&to)
            {
                continue;
            }

            let color_edge = self.next_edge_color();
            dot.add_edge(
                &self.node_ref(from),
                &self.node_ref(to),
                &[
                    ("lhead", &self.lref(to)),
                    ("ltail", &self.lref(from)),
                    ("tooltip", &self.edge_tooltip(from, to)),
                    ("color", color_edge),
                ],
            );
        }
    }

    /// Add ambiguity edges
    fn add_ambiguities(&self, dot: &mut DotGraph) {
        let mut conflicting_systems = self.graph.conflicting_systems().to_vec();
        conflicting_systems.sort();

        for (system_a, system_b, conflicts) in conflicting_systems {
            if !self
                .included_systems_sets
                .contains(&NodeId::System(system_a))
                || !self
                    .included_systems_sets
                    .contains(&NodeId::System(system_b))
            {
                continue;
            }

            if conflicts.is_empty() && !self.settings.ambiguity_enable_on_world {
                continue;
            }

            if let Some(include_ambiguity) = &self.settings.include_ambiguity {
                let system_a = self.graph.systems.get(system_a).unwrap();
                let system_b = self.graph.systems.get(system_b).unwrap();
                if !include_ambiguity(&system_a.system, &system_b.system, &conflicts, self.world) {
                    continue;
                }
            }

            let label = if conflicts.is_empty() {
                "World".to_owned()
            } else {
                let component_names = conflicts.iter().map(|&component_id| {
                    let component_name = self
                        .world
                        .components()
                        .get_info(component_id)
                        .unwrap()
                        .name();
                    let pretty_name = disqualified::ShortName(component_name.as_ref());

                    format!(
                        r#"<tr><td bgcolor="{}">{}</td></tr>"#,
                        self.settings.style.ambiguity_bgcolor,
                        crate::dot::html_escape(&pretty_name.to_string())
                    )
                });
                let trs = component_names.collect::<String>();
                format!(r#"RAW:<<table border="0" cellborder="0">{trs}</table>>"#)
            };

            dot.add_edge(
                &self.system_node_ref(system_a),
                &self.system_node_ref(system_b),
                &[
                    ("dir", "none"),
                    ("constraint", "false"),
                    ("color", &self.settings.style.ambiguity_color),
                    ("fontcolor", &self.settings.style.ambiguity_color),
                    ("label", &label),
                    (
                        "labeltooltip",
                        &self.edge_tooltip_undirected(
                            NodeId::System(system_a),
                            NodeId::System(system_b),
                        ),
                    ),
                ],
            );
        }
    }
}

impl ScheduleGraphContext<'_> {
    fn add_set_in_multiple_sets(&self, dot: &mut DotGraph, set_id: NodeId, set: &dyn SystemSet) {
        assert!(self.included_systems_sets.contains(&set_id));
        self.add_set(set_id, set, dot);

        for parent in hierarchy_parents(set_id, self.graph) {
            assert!(self.included_systems_sets.contains(&parent));
            dot.add_edge(
                &self.node_ref(parent),
                &self.node_ref(set_id),
                &[
                    ("dir", "none"),
                    ("color", &self.settings.style.multiple_set_edge_color),
                    ("ltail", &self.lref(parent)),
                    ("lhead", &self.lref(set_id)),
                ],
            );
        }
    }

    // add regular set and system hierarchy
    fn add_set(&self, set_id: NodeId, set: &dyn SystemSet, dot: &mut DotGraph) {
        let name = format!("{set:?}");

        if self.collapsed_sets.contains(&set_id) {
            dot.add_node(
                &node_index_name(set_id),
                &[("label", &name), ("tooltip", &name)],
            );

            return;
        }

        let system_set_cluster_name = node_index_name(set_id); // in sync with system_cluster_name
        let mut system_set_graph = DotGraph::subgraph(
            &system_set_cluster_name,
            &[
                ("style", "rounded,filled"),
                ("label", &name),
                ("tooltip", &name),
                ("fillcolor", &self.settings.style.color_set),
                ("fontcolor", &self.settings.style.color_set_label),
                ("color", &self.settings.style.color_set_border),
                ("penwidth", "2"),
            ],
        );

        system_set_graph.add_invisible_node(&marker_name(set_id));

        for &(nested_set_id, nested_set) in self
            .sets_in_single_set
            .get(&set_id)
            .map(|sets| sets.as_slice())
            .unwrap_or(&[])
        {
            self.add_set(nested_set_id, nested_set, &mut system_set_graph);
        }

        for &(nested_set_id, nested_set) in self
            .sets_in_multiple_sets
            .get(&Some(set_id))
            .map(|vec| vec.as_slice())
            .unwrap_or_default()
        {
            self.add_set_in_multiple_sets(&mut system_set_graph, nested_set_id, nested_set);
        }

        let systems = self
            .systems_in_single_set
            .get(&set_id)
            .map(|systems| systems.as_slice())
            .unwrap_or(&[]);

        for &(system_id, system) in systems.iter() {
            let name = self.system_name(system);
            let node_style = self.settings.get_system_style(system);

            system_set_graph.add_node(
                &self.node_ref(system_id),
                &[
                    ("label", &name),
                    ("tooltip", &system.name()),
                    ("fillcolor", &node_style.bg_color),
                    ("fontname", &self.settings.style.fontname),
                    ("fontcolor", &node_style.text_color),
                    ("color", &node_style.border_color),
                    ("penwidth", &node_style.border_width.to_string()),
                ],
            );
        }

        for &(system_id, system) in self
            .systems_in_multiple_sets
            .get(&Some(set_id))
            .map(|vec| vec.as_slice())
            .unwrap_or_default()
        {
            self.add_system_in_multiple_sets(&mut system_set_graph, system_id, system);
        }

        dot.add_sub_graph(system_set_graph);
    }

    fn add_system_in_multiple_sets(
        &self,
        dot: &mut DotGraph,
        system_id: NodeId,
        system: &ScheduleSystem,
    ) {
        assert!(self.included_systems_sets.contains(&system_id));
        let mut name = self.system_name(system);
        let name = name.to_mut();
        name.push_str("\nIn multiple sets");

        for parent in hierarchy_parents(system_id, self.graph) {
            assert!(self.included_systems_sets.contains(&parent));
            let parent_set = self
                .graph
                .system_sets
                .get(parent.as_set().unwrap())
                .unwrap();
            let _ = write!(name, ", {parent_set:?}");

            dot.add_edge(
                &self.node_ref(system_id),
                &self.node_ref(parent),
                &[
                    ("dir", "none"),
                    ("color", &self.settings.style.multiple_set_edge_color),
                    ("lhead", &set_cluster_name(parent)),
                ],
            );
        }
        dot.add_node(
            &self.node_ref(system_id),
            &[("label", name), ("tooltip", &system.name())],
        );
    }

    fn edge_tooltip(&self, a: NodeId, b: NodeId) -> String {
        format!("{} → {}", self.full_name(a), self.full_name(b))
    }

    fn edge_tooltip_undirected(&self, a: NodeId, b: NodeId) -> String {
        format!("{} — {}", self.full_name(a), self.full_name(b))
    }

    fn next_edge_color(&self) -> &str {
        use std::sync::atomic::Ordering;
        let (Ok(idx) | Err(idx)) =
            self.color_edge_idx
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |a| {
                    Some((a + 1) % self.settings.style.color_edge.len())
                });

        &self.settings.style.color_edge[idx]
    }
}

fn included_systems_sets(graph: &ScheduleGraph, settings: &Settings) -> HashSet<NodeId> {
    let Some(include_system) = &settings.include_system else {
        return graph
            .systems
            .iter()
            .map(|(id, ..)| NodeId::System(id))
            .chain(graph.system_sets.iter().map(|(id, ..)| NodeId::Set(id)))
            .collect();
    };

    let hierarchy = graph.hierarchy().graph();

    let root_sets = hierarchy.nodes().filter(|&node| {
        let Some(set_key) = node.as_set() else {
            return false;
        };
        graph.system_sets[set_key].system_type().is_none()
            && hierarchy
                .neighbors_directed(node, Direction::Incoming)
                .next()
                .is_none()
    });

    let systems_of_interest: HashSet<NodeId> = graph
        .systems
        .iter()
        .filter(|&(_, system, _)| include_system(system))
        .map(|(id, ..)| NodeId::System(id))
        .collect();

    fn include_ancestors(
        id: NodeId,
        hierarchy: &DiGraph<NodeId>,
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
        for &(a, b, ref conflicts) in &graph.conflicting_systems().0 {
            if !systems_of_interest.contains(&NodeId::System(a))
                || !systems_of_interest.contains(&NodeId::System(b))
            {
                continue;
            }

            if !settings.ambiguity_enable_on_world && conflicts.is_empty() {
                continue;
            }

            included_systems_sets.insert(NodeId::System(a));
            included_systems_sets.insert(NodeId::System(b));
        }
    }

    for (from, to) in graph.dependency().graph().all_edges() {
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

impl ScheduleGraphContext<'_> {
    fn system_name(&self, system: &ScheduleSystem) -> Cow<'_, str> {
        (*self.settings.system_name)(system).into()
    }

    fn system_set_name(&self, system_set: &dyn SystemSet) -> Cow<'_, str> {
        (*self.settings.system_set_name)(system_set).into()
    }

    fn full_name(&self, node_id: NodeId) -> Cow<'_, str> {
        match node_id {
            NodeId::System(key) => self.system_name(&self.graph.systems.get(key).unwrap().system),
            NodeId::Set(key) => self.system_set_name(self.graph.system_sets.get(key).unwrap()),
        }
    }

    fn is_non_system_set(&self, node_id: NodeId) -> bool {
        let NodeId::Set(key) = node_id else {
            return false;
        };
        self.graph
            .system_sets
            .get(key)
            .unwrap()
            .system_type()
            .is_none()
    }

    // lhead/ltail
    fn lref(&self, node_id: NodeId) -> String {
        if self.is_non_system_set(node_id) && !self.collapsed_sets.contains(&node_id) {
            set_cluster_name(node_id)
        } else {
            String::new()
        }
    }

    // PERF: O(n)
    fn system_of_system_type(&self, set: &dyn SystemSet) -> Option<SystemKey> {
        self.graph.systems.iter().find_map(|(id, system, _)| {
            if system.name().starts_with("print_schedule_graph") {
                dbg!(&system, system.default_system_sets(), set);
            }
            let is_system_set = system.default_system_sets().iter().any(|s| s.0 == set);
            is_system_set.then_some(id)
        })
    }

    fn system_node_ref(&self, node_id: SystemKey) -> String {
        if let Some(collapsed_set) = self.collapsed_set_children.get(&NodeId::System(node_id)) {
            node_index_name(*collapsed_set)
        } else {
            node_index_name(NodeId::System(node_id))
        }
    }

    fn node_ref(&self, node_id: NodeId) -> String {
        match node_id {
            NodeId::System(system) => self.system_node_ref(system),
            NodeId::Set(_) if self.collapsed_sets.contains(&node_id) => node_index_name(node_id),
            NodeId::Set(set) => {
                let set = self.graph.system_sets.get(set).unwrap();

                if set.system_type() == Some(TypeId::of::<ApplyDeferred>()) {
                    "ApplyDeferred".to_owned()
                } else if set.system_type().is_some() {
                    let system_node = self.system_of_system_type(set);
                    if let Some(system_node) = system_node {
                        self.system_node_ref(system_node)
                    } else {
                        let name = format!("{:?}", disqualified::ShortName(&format!("{set:?}")));
                        if name.starts_with("SystemTypeSet(fn FunctionSystem") {
                            let fn_name = name
                                .trim_end_matches(">())")
                                .rsplit_once(' ')
                                .unwrap()
                                .1
                                .to_owned();

                            format!("<missing> {fn_name}")
                        } else {
                            format!(
                                "<missing> {:?}",
                                disqualified::ShortName(&format!("{set:?}"))
                            )
                        }
                    }
                } else {
                    marker_name(node_id)
                }
            }
        }
    }
}

fn set_cluster_name(id: NodeId) -> String {
    assert!(id.is_set());
    format!("cluster{}", node_index_name(id))
}

fn node_index_name(node_id: NodeId) -> String {
    format!("node_{node_id:?}")
}
fn marker_name(node_id: NodeId) -> String {
    assert!(node_id.is_set());
    format!("set_marker_node_{node_id:?}")
}

enum IterSingleResult<T> {
    Empty,
    Single(T),
    Multiple(Vec<T>),
}
fn iter_single<T>(mut iter: impl Iterator<Item = T>) -> IterSingleResult<T> {
    let Some(first) = iter.next() else {
        return IterSingleResult::Empty;
    };
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

fn hierarchy_parents(node: NodeId, graph: &ScheduleGraph) -> impl Iterator<Item = NodeId> + '_ {
    let hierarchy = graph.hierarchy().graph();
    hierarchy
        .neighbors_directed(node, Direction::Incoming)
        .filter(|&parent| {
            let parent = parent.as_set().unwrap(); // TODO: why?
            graph
                .system_sets
                .get(parent)
                .unwrap()
                .system_type()
                .is_none()
        })
}

fn lowest_common_ancestor(parents: &[NodeId], hierarchy: &DiGraph<NodeId>) -> Option<NodeId> {
    let parent = parents.last().unwrap();
    let mut common_ancestors: Vec<_> = ancestors_of_node(*parent, hierarchy).collect();

    // PERF: O(depth*depth) but depth is probably always < 5
    for &other_parent in parents[0..parents.len() - 1].iter().rev() {
        common_ancestors.retain(|&ancestor| {
            ancestors_of_node(other_parent, hierarchy)
                .any(|other_ancestor| other_ancestor == ancestor)
        })
    }

    let first_common_ancestor = common_ancestors.first().copied();
    first_common_ancestor
}

fn ancestors_of_node(
    node_id: NodeId,
    graph: &DiGraph<NodeId>,
) -> impl Iterator<Item = NodeId> + '_ {
    let mut queue = VecDeque::with_capacity(1);
    queue.push_back(node_id);
    Ancestors { queue, graph }
}

struct Ancestors<'a> {
    queue: VecDeque<NodeId>,
    graph: &'a DiGraph<NodeId>,
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

fn collect_reachable(
    reachable: &mut HashSet<NodeId>,
    graph: &DiGraph<NodeId>,
    u: NodeId,
    direction: Direction,
) {
    for neighbor in graph.neighbors_directed(u, direction) {
        reachable.insert(neighbor);
        collect_reachable(reachable, graph, neighbor, direction);
    }
}

fn toposort(graph: &DiGraph<NodeId>) -> Vec<NodeId> {
    let mut visited = HashSet::new();
    let mut stack = Vec::new();

    fn dfs(
        visited: &mut HashSet<NodeId>,
        stack: &mut Vec<NodeId>,
        graph: &DiGraph<NodeId>,
        node: NodeId,
    ) {
        if !visited.insert(node) {
            return;
        }

        for neighbour in graph.neighbors(node) {
            dfs(visited, stack, graph, neighbour);
        }

        stack.push(node);
    }

    for node in graph.nodes() {
        dfs(&mut visited, &mut stack, graph, node);
    }

    stack.reverse();
    stack
}

fn remove_transitive_edges(graph: &mut DiGraph<NodeId>) {
    let toposort = toposort(graph);

    let mut reachable = HashSet::new();

    for visiting in toposort {
        let direct_heighbours: Vec<NodeId> = graph.neighbors(visiting).collect();
        for n in direct_heighbours {
            graph.remove_edge(visiting, n);

            reachable.clear();
            collect_reachable(&mut reachable, graph, visiting, Direction::Outgoing);

            // if we still can access a neighbour with a longer path, it's a transitive dependency.
            if !reachable.contains(&n) {
                // No longer path, so we're keeping that edge.
                graph.add_edge(visiting, n);
            }
        }
    }
}
