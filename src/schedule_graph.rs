use crate::{dot, dot::DotGraph};
use bevy_app::{App, AppLabel, StartupSchedule};
use bevy_ecs::{
    component::ComponentId,
    prelude::*,
    schedule::{GraphNode, StageLabelId, SystemContainer, SystemLabelId},
};
use pretty_type_name::pretty_type_name_str;

/// Formats the schedule into a dot graph.
///
/// By default, the `Startup` subschedule is not shown, to enable it use [`schedule_graph_dot_styled`] and enable [`ScheduleGraphStyle::hide_startup_schedule`].
pub fn schedule_graph_dot(schedule: &App) -> String {
    let default_style = ScheduleGraphStyle::dark();
    schedule_graph_dot_styled(schedule, &default_style)
}

#[non_exhaustive]
pub struct SystemInfo<'a> {
    pub name: &'a str,
}

pub struct ScheduleGraphStyle {
    pub fontsize: f32,
    pub fontname: String,
    pub bgcolor: String,
    pub bgcolor_nested_schedule: String,
    pub bgcolor_stage: String,
    pub color_system: String,
    pub color_edge: String,
    pub hide_startup_schedule: bool,
    #[allow(clippy::type_complexity)]
    pub system_filter: Option<Box<dyn Fn(&SystemInfo) -> bool>>,
}
impl ScheduleGraphStyle {
    pub fn light() -> Self {
        ScheduleGraphStyle {
            fontsize: 16.0,
            fontname: "Helvetica".into(),
            bgcolor: "white".into(),
            bgcolor_nested_schedule: "#d1d5da".into(),
            bgcolor_stage: "#e1e5ea".into(),
            color_system: "white".into(),
            color_edge: "black".into(),
            hide_startup_schedule: true,
            system_filter: None,
        }
    }
    pub fn dark() -> Self {
        ScheduleGraphStyle {
            fontsize: 16.0,
            fontname: "Helvetica".into(),
            bgcolor: "#35393F".into(),
            bgcolor_nested_schedule: "#D0E1ED".into(),
            bgcolor_stage: "#99aab5".into(),
            color_system: "#eff1f3".into(),
            color_edge: "white".into(),
            hide_startup_schedule: true,
            system_filter: None,
        }
    }
}
impl Default for ScheduleGraphStyle {
    fn default() -> Self {
        ScheduleGraphStyle::dark()
    }
}

/// Formats the schedule into a dot graph using a custom [`ScheduleGraphStyle`].
pub fn schedule_graph_dot_styled(app: &App, style: &ScheduleGraphStyle) -> String {
    schedule_graph_dot_styled_inner(app, None, style)
}

/// Formats the schedule of a sub app into a dot graph using a custom [`ScheduleGraphStyle`].
///
/// Additionally accepts an array of stages for which
/// the main world should be used when resolving system access for tooltips.
/// This is useful for the render app, where the `Extract` stage is run on
/// the main app but the command queue are applied on the render app.
pub fn schedule_graph_dot_sub_app_styled(
    app: &App,
    label: impl AppLabel,
    stages_using_main_world: &[&dyn StageLabel],
    style: &ScheduleGraphStyle,
) -> String {
    schedule_graph_dot_styled_inner(
        app.sub_app(label),
        Some((&app.world, stages_using_main_world)),
        style,
    )
}

fn schedule_graph_dot_styled_inner(
    app: &App,
    use_world_info_for_stages: Option<(&World, &[&dyn StageLabel])>,
    style: &ScheduleGraphStyle,
) -> String {
    let mut graph = DotGraph::new(
        "schedule",
        "digraph",
        &[
            ("fontsize", &style.fontsize.to_string()),
            ("fontname", &style.fontname),
            ("rankdir", "LR"),
            ("nodesep", "0.05"),
            ("bgcolor", &style.bgcolor),
            ("compound", "true"),
        ],
    )
    .node_attributes(&[("shape", "box"), ("margin", "0"), ("height", "0.4")])
    .edge_attributes(&[("color", &style.color_edge)]);

    build_schedule_graph(
        &mut graph,
        app,
        &app.schedule,
        "schedule",
        None,
        use_world_info_for_stages,
        style,
    );

    graph.finish()
}

fn build_schedule_graph(
    graph: &mut DotGraph,
    app: &App,
    schedule: &Schedule,
    schedule_name: &str,
    marker_node_id: Option<&str>,
    use_world_info_for_stages: Option<(&World, &[&dyn StageLabel])>,
    style: &ScheduleGraphStyle,
) {
    if let Some(marker_id) = marker_node_id {
        graph.add_invisible_node(marker_id);
    }

    let is_startup_schedule = |stage_name: StageLabelId| stage_name == StartupSchedule.as_label();

    for (stage_name, stage) in schedule.iter_stages() {
        if let Some(system_stage) = stage.downcast_ref::<SystemStage>() {
            let subgraph = system_stage_subgraph(
                &app.world,
                schedule_name,
                stage_name,
                system_stage,
                use_world_info_for_stages,
                style,
            );
            graph.add_sub_graph(subgraph);
        } else if let Some(schedule) = stage.downcast_ref::<Schedule>() {
            if style.hide_startup_schedule && is_startup_schedule(stage_name) {
                continue;
            }

            let name = format!("cluster_{:?}", stage_name);

            let marker_id = marker_id(schedule_name, stage_name);
            let stage_name_str = format!("{:?}", stage_name);

            let mut schedule_sub_graph = DotGraph::new(
                &name,
                "subgraph",
                &[
                    ("label", &stage_name_str),
                    ("fontsize", "20"),
                    ("constraint", "false"),
                    ("rankdir", "LR"),
                    ("style", "rounded"),
                    ("bgcolor", &style.bgcolor_nested_schedule),
                ],
            )
            .edge_attributes(&[("color", &style.color_edge)]);
            build_schedule_graph(
                &mut schedule_sub_graph,
                app,
                schedule,
                &name,
                Some(&marker_id),
                use_world_info_for_stages,
                style,
            );
            graph.add_sub_graph(schedule_sub_graph);
        } else {
            eprintln!("Missing downcast: {:?}", stage_name);
        }
    }

    let iter_a = schedule
        .iter_stages()
        .filter(|(stage, _)| !style.hide_startup_schedule || !is_startup_schedule(*stage));
    let iter_b = schedule
        .iter_stages()
        .filter(|(stage, _)| !style.hide_startup_schedule || !is_startup_schedule(*stage))
        .skip(1);

    for ((a, _), (b, _)) in iter_a.zip(iter_b) {
        let a = marker_id(schedule_name, a);
        let b = marker_id(schedule_name, b);
        graph.add_edge(&a, &b, &[]);
    }
}

fn marker_id(schedule_name: &str, stage_name: StageLabelId) -> String {
    format!("MARKER_{}_{:?}", schedule_name, stage_name)
}

fn system_stage_subgraph(
    world: &World,
    schedule_name: &str,
    stage_name: StageLabelId,
    system_stage: &SystemStage,
    use_world_info_for_stages: Option<(&World, &[&dyn StageLabel])>,
    style: &ScheduleGraphStyle,
) -> DotGraph {
    let stage_name_str = stage_name.as_str();

    let mut sub = DotGraph::new(
        &format!("cluster_{:?}", stage_name.as_str()),
        "subgraph",
        &[
            ("style", "rounded"),
            ("color", &style.bgcolor_stage),
            ("bgcolor", &style.bgcolor_stage),
            ("rankdir", "TD"),
            ("label", stage_name_str),
        ],
    )
    .node_attributes(&[
        ("style", "filled"),
        ("color", &style.color_system),
        ("bgcolor", &style.color_system),
    ]);

    sub.add_invisible_node(&marker_id(schedule_name, stage_name));

    let relevant_world = match use_world_info_for_stages {
        Some((relevant_world, stages))
            if stages.iter().any(|stage| stage.as_label() == stage_name) =>
        {
            relevant_world
        }
        _ => world,
    };

    add_systems_to_graph(
        &mut sub,
        relevant_world,
        schedule_name,
        SystemKind::ExclusiveStart,
        system_stage.exclusive_at_start_systems(),
        style,
    );
    add_systems_to_graph(
        &mut sub,
        relevant_world,
        schedule_name,
        SystemKind::ExclusiveBeforeCommands,
        system_stage.exclusive_before_commands_systems(),
        style,
    );
    add_systems_to_graph(
        &mut sub,
        relevant_world,
        schedule_name,
        SystemKind::Parallel,
        system_stage.parallel_systems(),
        style,
    );
    add_systems_to_graph(
        &mut sub,
        relevant_world,
        schedule_name,
        SystemKind::ExclusiveEnd,
        system_stage.exclusive_at_end_systems(),
        style,
    );

    sub
}

enum SystemKind {
    ExclusiveStart,
    ExclusiveEnd,
    ExclusiveBeforeCommands,
    Parallel,
}
fn add_systems_to_graph(
    graph: &mut DotGraph,
    world: &World,
    schedule_name: &str,
    kind: SystemKind,
    systems: &[SystemContainer],
    style: &ScheduleGraphStyle,
) {
    let mut systems: Vec<_> = systems.iter().collect();
    systems.sort_by_key(|system| system.name());

    if systems.is_empty() {
        return;
    }

    for (i, &system_container) in systems.iter().enumerate() {
        let id = node_id(schedule_name, system_container, i);
        let system_name = system_container.name();

        if let Some(filter) = &style.system_filter {
            let info = SystemInfo {
                name: system_name.as_ref(),
            };
            if !filter(&info) {
                continue;
            }
        }

        let short_system_name = pretty_type_name_str(&system_container.name());

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

        let tooltip = system_tooltip(system_container, world);
        graph.add_node(&id, &[("label", &label), ("tooltip", &tooltip)]);

        add_dependency_labels(
            graph,
            schedule_name,
            &id,
            SystemDirection::Before,
            system_container.before(),
            &systems,
        );
        add_dependency_labels(
            graph,
            schedule_name,
            &id,
            SystemDirection::After,
            system_container.after(),
            &systems,
        );
    }
}

fn system_tooltip(system_container: &SystemContainer, world: &World) -> String {
    let mut tooltip = String::new();
    let truncate_in_place =
        |tooltip: &mut String, end: &str| tooltip.truncate(tooltip.trim_end_matches(end).len());

    let components = world.components();
    let name_of_component = |id| {
        pretty_type_name_str(
            components
                .get_info(id)
                .map_or_else(|| "<missing>", |info| info.name()),
        )
    };

    let is_resource = |id: &ComponentId| world.storages().resources.get(*id).is_some();

    let component_access = system_container.component_access();
    let (read_resources, read_components): (Vec<_>, Vec<_>) =
        component_access.reads().partition(is_resource);
    let (write_resources, write_components): (Vec<_>, Vec<_>) =
        component_access.writes().partition(is_resource);

    let mut list = |name, components: &[ComponentId]| {
        if components.is_empty() {
            return;
        }
        tooltip.push_str(name);
        tooltip.push_str(" [");
        for read_resource in components {
            tooltip.push_str(&name_of_component(*read_resource));
            tooltip.push_str(", ");
        }
        truncate_in_place(&mut tooltip, ", ");
        tooltip.push_str("]\\n");
    };

    list("Components", &read_components);
    list("ComponentsMut", &write_components);

    list("Res", &read_resources);
    list("ResMut", &write_resources);

    if tooltip.is_empty() {
        pretty_type_name_str(&system_container.name())
    } else {
        tooltip
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
    requirements: &[SystemLabelId],
    other_systems: &[&SystemContainer],
) {
    for requirement in requirements {
        let mut found = false;
        for (i, &dependency) in other_systems
            .iter()
            .enumerate()
            .filter(|(_, node)| node.labels().contains(requirement))
        {
            found = true;

            let me = system_node_id;
            let other = node_id(schedule_name, dependency, i);

            match direction {
                SystemDirection::Before => graph.add_edge(me, &other, &[("constraint", "false")]),
                SystemDirection::After => graph.add_edge(&other, me, &[("constraint", "false")]),
            }
        }
        assert!(found);
    }
}

fn node_id(schedule_name: &str, system: &SystemContainer, i: usize) -> String {
    format!("{}_{}_{}", schedule_name, system.name(), i)
}
