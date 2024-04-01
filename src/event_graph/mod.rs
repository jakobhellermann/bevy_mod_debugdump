pub mod settings;
mod system_style;

use std::sync::atomic::AtomicUsize;

use bevy_ecs::{
    component::ComponentId,
    schedule::{NodeId, Schedule, ScheduleLabel, Schedules},
    world::World,
};
use bevy_utils::hashbrown::{HashMap, HashSet};

use crate::dot::DotGraph;
pub use settings::Settings;

pub struct EventGraphContext<'a> {
    settings: &'a Settings,

    events_tracked: HashSet<ComponentId>,
    event_readers: HashMap<ComponentId, Vec<NodeId>>,
    event_writers: HashMap<ComponentId, Vec<NodeId>>,
    schedule: Box<dyn ScheduleLabel>,

    color_edge_idx: AtomicUsize,
}

impl<'a> EventGraphContext<'a> {
    fn next_edge_color(&self) -> &str {
        use std::sync::atomic::Ordering;
        let (Ok(idx) | Err(idx)) =
            self.color_edge_idx
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |a| {
                    Some((a + 1) % self.settings.style.color_edge.len())
                });

        &self.settings.style.color_edge[idx]
    }

    fn add_system(
        &self,
        dot: &mut DotGraph,
        system: &NodeId,
        graph: &bevy_ecs::schedule::ScheduleGraph,
    ) {
        let sys = &graph.get_system_at(*system).unwrap();
        let name = &sys.name();

        let node_style = self.settings.get_system_style(*sys);
        dot.add_node(
            name,
            &[
                ("label", &display_name(name, self.settings)),
                ("tooltip", name),
                ("shape", "box"),
                ("fillcolor", &node_style.bg_color),
                ("fontname", &self.settings.style.fontname),
                ("fontcolor", &node_style.text_color),
                ("color", &node_style.border_color),
                ("penwidth", &node_style.border_width.to_string()),
            ],
        );
    }

    fn add_event(&self, dot: &mut DotGraph, event: &ComponentId, world: &World) -> String {
        let component = world.components().get_info(*event).unwrap();
        // Relevant name is only what's inside "bevy::ecs::Events<(...)>"
        let full_name = component.name();
        let name = full_name.split_once('<').unwrap().1;
        let name = &name[0..name.len() - 1];
        let event_id = format!("event_{0}", event.index());
        let node_style = self.settings.get_event_style(component);
        dot.add_node(
            &event_id,
            &[
                ("label", &display_name(name, self.settings)),
                ("tooltip", name),
                ("shape", "ellipse"),
                ("fillcolor", &node_style.bg_color),
                ("fontname", &self.settings.style.fontname),
                ("fontcolor", &node_style.text_color),
                ("color", &node_style.border_color),
                ("penwidth", &node_style.border_width.to_string()),
            ],
        );
        event_id
    }
}

/// Formats the events into a dot graph.
pub fn events_graph_dot<'a>(
    schedule: &Schedule,
    world: &World,
    settings: &'a Settings,
) -> EventGraphContext<'a> {
    let graph = schedule.graph();

    let mut events_tracked = HashSet::new();
    let mut event_readers = HashMap::<ComponentId, Vec<NodeId>>::new();
    let mut event_writers = HashMap::<ComponentId, Vec<NodeId>>::new();
    for (system_id, system, _condition) in graph.systems() {
        if let Some(include_system) = &settings.include_system {
            if !(include_system)(system) {
                continue;
            }
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
        settings,
        events_tracked,
        event_readers,
        event_writers,
        schedule: Box::new(schedule.label()),
        color_edge_idx: AtomicUsize::new(0),
    }
}

pub fn print_only_context(
    schedules: &bevy_ecs::schedule::Schedules,
    dot: &mut DotGraph,
    ctx: &EventGraphContext,
    world: &World,
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
        ctx.add_system(dot, s, graph);
    }

    for event in ctx.events_tracked.iter() {
        let readers = ctx.event_readers.get(event).cloned().unwrap_or_default();
        let writers = ctx.event_writers.get(event).cloned().unwrap_or_default();

        let event_id = ctx.add_event(dot, event, world);

        for writer in writers {
            // We have to use full names, because nodeId is schedule specific, and I want to support multiple schedules displayed
            let system_name = graph.get_system_at(writer).unwrap().name();
            dot.add_edge(
                &system_name,
                &event_id,
                &[
                    // TODO: customize edges, colors in a same fashion as schedules
                    /*
                    ("lhead", &self.lref(to)),
                    ("ltail", &self.lref(from)),
                    ("tooltip", &self.edge_tooltip(from, to)),
                     */
                    ("color", ctx.next_edge_color()),
                ],
            );
        }
        for reader in readers {
            let system_name = graph.get_system_at(reader).unwrap().name();

            dot.add_edge(
                &event_id,
                &system_name,
                &[
                    /*
                    ("lhead", &self.lref(to)),
                    ("ltail", &self.lref(from)),
                    ("tooltip", &self.edge_tooltip(from, to)),
                    */
                    ("color", ctx.next_edge_color()),
                ],
            );
        }
    }
}

fn display_name(name: &str, settings: &Settings) -> String {
    if settings.prettify_system_names {
        pretty_type_name::pretty_type_name_str(name)
    } else {
        name.to_string()
    }
}

pub fn print_context(
    schedules: &Schedules,
    ctxs: &Vec<EventGraphContext>,
    world: &World,
    settings: &Settings,
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
        print_only_context(schedules, &mut dot, ctx, world);
    }
    dot.finish().to_string()
}
