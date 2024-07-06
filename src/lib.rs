use std::io::Write;
use std::{fs::File, path::PathBuf};

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

            let label_name = format!("{:?}", label);
            let schedule = schedules
                .get_mut(label)
                .ok_or_else(|| format!("schedule {label_name} doesn't exist"))
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

/// Check the command line for arguments relevant to this crate.
///
/// ## Dump the render graph
///
/// Use `--dump-render <file.dot>` to dump the render graph.
///
/// ## Dump the schedule graph
///
/// Use `--dump-update-schedule <file.dot>` to dump the `Update` schedule graph.
///
/// ## Exit the app
///
/// Use `--exit` to exit the app. This may be useful if one wants to create
/// these graphs in script.
///
/// # Usage
///
/// Set up your app as usual. No log disabling required. Add the
/// `bevy_mod_debugdump::CommandLineArgs` plugin at the end. And run your app.
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
/// TODO: Consider adding a means of selecting a schedule other than `Update`.
pub struct CommandLineArgs;

struct Args {
    dump_render: Option<PathBuf>,
    dump_update_schedule: Option<PathBuf>,
    exit: bool,
}

fn parse_args() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;

    let mut dump_update_schedule = None;
    let mut dump_render = None;
    let mut exit = true;

    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Long("dump-update-schedule") => dump_update_schedule = Some(parser.value()?.parse()?),
            Long("dump-render") => dump_render = Some(parser.value()?.parse()?),
            Long("no-exit") => exit = false,
            Long("help") => {
                println!("Usage: [--dump-update-schedule file] [--dump-render file] [--no-exit]");
                std::process::exit(0);
            }
            _ => return Err(arg.unexpected()),
        }
    }

    Ok(Args {
        dump_render,
        dump_update_schedule,
        exit,
    })
}

fn execute_cli(app: &mut App) -> Result<Args, Box<dyn std::error::Error>> {
    let args = parse_args()?;

    if let Some(dump_render) = &args.dump_render {
        let settings = render_graph::Settings::default();
        let mut out = File::create(dump_render)?;
        write!(out, "{}", render_graph_dot(app, &settings))?;
    }

    if let Some(dump_update_schedule) = &args.dump_update_schedule {
        let settings = schedule_graph::Settings::default();
        let mut out = File::create(dump_update_schedule)?;
        write!(
            out,
            "{}",
            schedule_graph_dot(app, bevy_app::Update, &settings)
        )?;
    }

    Ok(args)
}

impl bevy_app::Plugin for CommandLineArgs {
    fn build(&self, app: &mut App) {
        let exit = match execute_cli(app) {
            Ok(args) => args.exit,
            Err(e) => {
                eprintln!("{e:?}");
                true
            }
        };

        if exit {
            // TODO: It would be nice if we could exit before the window
            // opens, but I don't see how.
            app.add_systems(
                bevy_app::First,
                |mut app_exit_events: bevy_ecs::event::EventWriter<bevy_app::AppExit>| {
                    app_exit_events.send(bevy_app::AppExit::Success);
                },
            );
        }
    }
}
