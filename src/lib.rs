use bevy_app::{App, Main};
use bevy_ecs::schedule::{ScheduleLabel, Schedules};

mod dot;

#[cfg(feature = "render_graph")]
pub mod render_graph;
pub mod schedule_graph;

/// Formats the schedule into a dot graph.
#[track_caller]
pub fn schedule_graph_dot(
    app: &mut App,
    label: impl ScheduleLabel,
    settings: &schedule_graph::Settings,
) -> String {
    app.world
        .resource_scope::<Schedules, _>(|world, mut schedules| {
            let schedule = schedules
                .get_mut(&label)
                .ok_or_else(|| format!("schedule with label {label:?} doesn't exist"))
                .unwrap();
            schedule.graph_mut().initialize(world);
            let _ = schedule.graph_mut().build_schedule(world.components());

            schedule_graph::schedule_graph_dot(schedule, world, &settings)
        })
}

/// Prints the [`CoreSchedule::Main`] with default settings.
pub fn print_main_schedule(app: &mut App) {
    let dot = schedule_graph_dot(app, Main, &schedule_graph::Settings::default());
    println!("{dot}");
}

/// Returns the current render graph using [`render_graph_dot`](render_graph::render_graph_dot).
/// # Example
/// ```rust,no_run
/// use bevy::prelude::*;
///
/// fn main() {
///     let mut app = App::new();
///     app.add_plugins(DefaultPlugins);
///     let settings = bevy_mod_debugdump::render_graph::Settings::default();
///     let dot = bevy_mod_debugdump::render_graph_dot(&mut app, &settings);
///     println!("{dot}");
/// }
/// ```
#[cfg(feature = "render_graph")]
pub fn render_graph_dot(app: &App, settings: &render_graph::Settings) -> String {
    use bevy_render::render_graph::RenderGraph;

    let render_app = app
        .get_sub_app(bevy_render::RenderApp)
        .unwrap_or_else(|_| panic!("no render app"));
    let render_graph = render_app.world.get_resource::<RenderGraph>().unwrap();

    render_graph::render_graph_dot(render_graph, &settings)
}

/// Prints the current render graph using [`render_graph_dot`](render_graph::render_graph_dot).
/// # Example
/// ```rust,no_run
/// use bevy::prelude::*;
///
/// fn main() {
///     let mut app = App::new();
///     app.add_plugins(DefaultPlugins);
///     bevy_mod_debugdump::print_render_graph(&mut app);
/// }
/// ```
#[cfg(feature = "render_graph")]
pub fn print_render_graph(app: &mut App) {
    let dot = render_graph_dot(app, &render_graph::Settings::default());
    println!("{dot}");
}
