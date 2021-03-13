use crate::{dot, dot::DotGraph};
use bevy::ecs::{prelude::*, schedule::SystemContainer};

pub fn schedule_graph_dot(schedule: &Schedule) -> String {
    schedule_graph(
        schedule,
        "schedule",
        "digraph",
        &[
            "rankdir=\"LR\"",
            "nodesep=0.05",
            "node [shape=box, margin=0, height=0.4]",
        ],
        None,
    )
    .finish()
}

fn schedule_graph(
    schedule: &Schedule,
    schedule_name: &str,
    kind: &str,
    options: &[&str],
    marker_node_id: Option<&str>,
) -> DotGraph {
    let mut graph = DotGraph::new(schedule_name, kind, options);

    if let Some(marker_id) = marker_node_id {
        graph.add_invisible_node(marker_id);
    }

    for (stage_name, stage) in schedule.iter_stages() {
        if let Some(system_stage) = stage.downcast_ref::<SystemStage>() {
            let subgraph = system_stage_subgraph(schedule_name, stage_name, system_stage);

            graph.add_sub_graph(subgraph);
        } else if let Some(schedule) = stage.downcast_ref::<Schedule>() {
            let name = format!("cluster_{:?}", stage_name);
            let label = format!("label=\"{:?}\"", stage_name);

            let marker_id = marker_id(&schedule_name, stage_name);

            let subgraph = schedule_graph(
                schedule,
                &name,
                "subgraph",
                &[&label, "constraint=false", "rankdir=\"LR\""],
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
    let label = format!(r#"label = "{:?}""#, stage_name);
    let mut sub = DotGraph::new(
        &format!("cluster_{:?}", stage_name),
        "subgraph",
        &[
            "style=filled",
            "color=lightgrey",
            "node [style=filled,color=white]",
            "rankdir=\"TD\"",
            &label,
        ],
    );

    sub.add_invisible_node(&marker_id(schedule_name, stage_name));

    add_systems_to_graph(
        &mut sub,
        SystemKind::ExclusiveStart,
        system_stage.exclusive_at_start_systems(),
    );
    add_systems_to_graph(
        &mut sub,
        SystemKind::ExclusiveBeforeCommands,
        system_stage.exclusive_before_commands_systems(),
    );
    add_systems_to_graph(
        &mut sub,
        SystemKind::Parallel,
        system_stage.parallel_systems(),
    );
    add_systems_to_graph(
        &mut sub,
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
fn add_systems_to_graph<T: SystemContainer>(graph: &mut DotGraph, kind: SystemKind, systems: &[T]) {
    if systems.is_empty() {
        return;
    }

    for system_container in systems {
        let system_name = system_container.name();
        let short_system_name = pretty_type_name::pretty_type_name_str(&system_name);

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
            None => quote(&short_system_name),
        };

        graph.add_node(
            &quote(&system_name),
            // &[("label", &quote(&short_system_name))],
            &[("label", &label)],
        );

        add_dependency_labels(
            graph,
            &system_name,
            SystemDirection::Before,
            system_container.before(),
            systems,
        );
        add_dependency_labels(
            graph,
            &system_name,
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
    system_name: &str,
    direction: SystemDirection,
    requirements: &[Box<dyn SystemLabel>],
    other_systems: &[impl SystemContainer],
) {
    for requirement in requirements {
        let mut found = false;
        for node in other_systems
            .iter()
            .filter(|node| node.labels().contains(requirement))
        {
            found = true;

            let before_id = node.name();
            let me = quote(system_name);
            let other = quote(&before_id);

            match direction {
                SystemDirection::Before => graph.add_edge(&me, &other, &[("constraint", "false")]),
                SystemDirection::After => graph.add_edge(&other, &me, &[("constraint", "false")]),
            }
        }
        assert!(found);
    }
}

fn quote(str: &str) -> String {
    format!("\"{}\"", str)
}
