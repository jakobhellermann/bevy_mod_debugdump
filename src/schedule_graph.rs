use crate::{dot, dot::DotGraph};
use bevy::ecs::{prelude::*, schedule::SystemContainer};

/// Formats the schedule into a dot graph.
pub fn schedule_graph_dot(schedule: &Schedule) -> String {
    schedule_graph(
        schedule,
        "schedule",
        "digraph",
        &[("rankdir", "LR"), ("nodesep", "0.05")],
        &[("shape", "box"), ("margin", "0"), ("height", "0.4")],
        None,
    )
    .finish()
}

fn schedule_graph(
    schedule: &Schedule,
    schedule_name: &str,
    kind: &str,
    attrs: &[(&str, &str)],
    node_attrs: &[(&str, &str)],
    marker_node_id: Option<&str>,
) -> DotGraph {
    let mut graph = DotGraph::new(schedule_name, kind, attrs).node_attributes(node_attrs);

    if let Some(marker_id) = marker_node_id {
        graph.add_invisible_node(marker_id);
    }

    for (stage_name, stage) in schedule.iter_stages() {
        if let Some(system_stage) = stage.downcast_ref::<SystemStage>() {
            let subgraph = system_stage_subgraph(schedule_name, stage_name, system_stage);
            graph.add_sub_graph(subgraph);
        } else if let Some(schedule) = stage.downcast_ref::<Schedule>() {
            let name = format!("cluster_{:?}", stage_name);

            let marker_id = marker_id(&schedule_name, stage_name);
            let stage_name_str = format!("{:?}", stage_name);

            let subgraph = schedule_graph(
                schedule,
                &name,
                "subgraph",
                &[
                    ("label", &stage_name_str),
                    ("constraint", "false"),
                    ("rankdir", "LR"),
                ],
                &[],
                Some(&marker_id),
            );
            graph.add_sub_graph(subgraph);
        } else {
            eprintln!("Missing downcast: {:?}", stage_name);
        }
    }

    for ((a, _), (b, _)) in schedule.iter_stages().zip(schedule.iter_stages().skip(1)) {
        let a = marker_id(schedule_name, a);
        let b = marker_id(schedule_name, b);
        graph.add_edge(&a, &b, &[]);
    }

    graph
}

fn marker_id(schedule_name: &str, stage_name: &dyn StageLabel) -> String {
    format!("MARKER_{}_{:?}", schedule_name, stage_name,)
}

fn system_stage_subgraph(
    schedule_name: &str,
    stage_name: &dyn StageLabel,
    system_stage: &SystemStage,
) -> DotGraph {
    let stage_name_str = format!("{:?}", stage_name);
    let mut sub = DotGraph::new(
        &format!("cluster_{:?}", stage_name),
        "subgraph",
        &[
            ("style", "filled"),
            ("color", "lightgrey"),
            ("rankdir", "TD"),
            ("label", &stage_name_str),
        ],
    )
    .node_attributes(&[("style", "filled"), ("color", "white")]);

    sub.add_invisible_node(&marker_id(schedule_name, stage_name));

    add_systems_to_graph(
        &mut sub,
        schedule_name,
        SystemKind::ExclusiveStart,
        system_stage.exclusive_at_start_systems(),
    );
    add_systems_to_graph(
        &mut sub,
        schedule_name,
        SystemKind::ExclusiveBeforeCommands,
        system_stage.exclusive_before_commands_systems(),
    );
    add_systems_to_graph(
        &mut sub,
        schedule_name,
        SystemKind::Parallel,
        system_stage.parallel_systems(),
    );
    add_systems_to_graph(
        &mut sub,
        schedule_name,
        SystemKind::ExclusiveEnd,
        system_stage.exclusive_at_end_systems(),
    );

    sub
}

enum SystemKind {
    ExclusiveStart,
    ExclusiveEnd,
    ExclusiveBeforeCommands,
    Parallel,
}
fn add_systems_to_graph<T: SystemContainer>(
    graph: &mut DotGraph,
    schedule_name: &str,
    kind: SystemKind,
    systems: &[T],
) {
    if systems.is_empty() {
        return;
    }

    for (i, system_container) in systems.iter().enumerate() {
        let id = node_id(schedule_name, system_container, i);
        let short_system_name = pretty_type_name::pretty_type_name_str(&system_container.name());

        let kind = match kind {
            SystemKind::ExclusiveStart => Some("Exclusive at start"),
            SystemKind::ExclusiveEnd => Some("Exclusive at end"),
            SystemKind::ExclusiveBeforeCommands => Some("Exclusive before commands"),
            SystemKind::Parallel => None,
        };

        let label = match kind {
            Some(kind) => {
                format!(
                    r#"<{}<BR />{}>"#,
                    &dot::html_escape(&short_system_name),
                    dot::font_tag(kind, "red", 11),
                )
            }
            None => short_system_name,
        };

        graph.add_node(&id, &[("label", &label)]);

        add_dependency_labels(
            graph,
            schedule_name,
            &id,
            SystemDirection::Before,
            system_container.before(),
            systems,
        );
        add_dependency_labels(
            graph,
            schedule_name,
            &id,
            SystemDirection::After,
            system_container.after(),
            systems,
        );
    }
}

enum SystemDirection {
    Before,
    After,
}
fn add_dependency_labels(
    graph: &mut DotGraph,
    schedule_name: &str,
    system_node_id: &str,
    direction: SystemDirection,
    requirements: &[Box<dyn SystemLabel>],
    other_systems: &[impl SystemContainer],
) {
    for requirement in requirements {
        let mut found = false;
        for (i, dependency) in other_systems
            .iter()
            .enumerate()
            .filter(|(_, node)| node.labels().contains(requirement))
        {
            found = true;

            let me = system_node_id;
            let other = node_id(schedule_name, dependency, i);

            match direction {
                SystemDirection::Before => graph.add_edge(&me, &other, &[("constraint", "false")]),
                SystemDirection::After => graph.add_edge(&other, &me, &[("constraint", "false")]),
            }
        }
        assert!(found);
    }
}

fn node_id(schedule_name: &str, system: &impl SystemContainer, i: usize) -> String {
    format!("\"{}_{}_{}\"", schedule_name, system.name(), i)
}
