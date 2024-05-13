use bevy_app::App;
use bevy_ecs::schedule::{ScheduleLabel, Schedules};

mod dot;

#[cfg(feature = "render_graph")]
pub mod render_graph;
pub mod schedule_graph;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct ScheduleDebugGroup;

/// Formats the schedule into a dot graph.
#[track_caller]
pub fn schedule_graph_dot(
    app: &mut App,
    label: impl ScheduleLabel,
    settings: &schedule_graph::Settings,
) -> String {
    app.world_mut()
        .resource_scope::<Schedules, _>(|world, mut schedules| {
            let ignored_ambiguities = schedules.ignored_scheduling_ambiguities.clone();

            let schedule = schedules
                .get_mut(label)
                .ok_or_else(|| "schedule doesn't exist".to_string())
                .unwrap();
            schedule.graph_mut().initialize(world);
            let _ = schedule.graph_mut().build_schedule(
                world.components(),
                ScheduleDebugGroup.intern(),
                &ignored_ambiguities,
            );

            schedule_graph::schedule_graph_dot(schedule, world, settings)
        })
}

/// Prints the schedule with default settings.
pub fn print_schedule_graph(app: &mut App, schedule_label: impl ScheduleLabel) {
    let dot = schedule_graph_dot(app, schedule_label, &schedule_graph::Settings::default());
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
        .unwrap_or_else(|| panic!("no render app"));
    let render_graph = render_app.world().get_resource::<RenderGraph>().unwrap();

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

/// Check the command line for arguments relevant to this crate. The app will
/// run as before.
///
/// # Dump the render graph
///
/// Use `--dump-render <file.dot>` to dump the render graph.
///
/// # Dump the schedule graph
///
/// Use `--dump-schedule <file.dot>` to dump the `Update` schedule graph.
///
/// Does not require disabling of logging.
///
/// ```rust,no_run
/// use bevy::prelude::*;
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         // Include all other setup as normal.
///         .add_plugins(bevy_mod_debugdump::CommandLineArgs)
///         .run();
/// }
/// ```
///
/// # Exit the app
///
/// Use `--exit` to exit the app. This may be useful if one wants to create
/// these graphs in script.
///
/// TODO: Consider adding a means of selecting a schedule other than `Update`.
pub struct CommandLineArgs;

impl bevy_app::Plugin for CommandLineArgs {
    fn build(&self, app: &mut App) {
        use std::fs::File;
        use std::io::Write;
        let mut args = std::env::args();
        while let Some(arg) = args.next() {
            if arg == "--dump-render" {
                let settings = render_graph::Settings::default();
                let mut out =
                    File::create(args.next().expect("file argument")).expect("file create");
                write!(out, "{}", render_graph_dot(app, &settings)).expect("write file");
            } else if arg == "--dump-schedule" {
                let settings = schedule_graph::Settings::default();
                let mut out =
                    File::create(args.next().expect("file argument")).expect("file create");
                write!(
                    out,
                    "{}",
                    schedule_graph_dot(app, bevy_app::Update, &settings)
                )
                .expect("write file");
            } else if arg == "--exit" {
                use bevy_ecs::event::EventWriter;
                // TODO: It would be nice if we could exit before the window
                // opens, but I don't see how.
                app.add_systems(bevy_app::First, |mut app_exit_events: EventWriter<bevy_app::AppExit>| {
                    app_exit_events.send(bevy_app::AppExit);
                });
            }
        }
    }
}
