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

/// Returns the current render graph using
/// [`render_graph_dot`](render_graph::render_graph_dot). # Example
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

/// Prints the current render graph using
/// [`render_graph_dot`](render_graph::render_graph_dot). # Example
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
/// Use `dump-render <file.dot>` to dump the render graph.
///
/// ## Dump the schedule graph
///
/// Use `dump-update-schedule <file.dot>` to dump the `Update` schedule graph.
///
/// ## Exit the app
///
/// By default the app will exit after performing the dump. If you want to keep
/// the app running, use `--no-exit`.
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
    command: ArgsCommand,
    exit: bool,
}

/// A command to execute from the CLI.
enum ArgsCommand {
    None,
    /// Dumps the render graph to the specified file path.
    DumpRender(PathBuf),
    /// Dumps the Update schedule graph to the specified file path.
    DumpUpdateSchedule(PathBuf),
}

fn parse_args() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;

    let mut command = ArgsCommand::None;
    let mut exit = true;

    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match &arg {
            Value(value) => {
                if !matches!(command, ArgsCommand::None) {
                    return Err(arg.unexpected());
                }

                if value == "dump-update-schedule" {
                    let path = parser.value()?.parse()?;
                    command = ArgsCommand::DumpUpdateSchedule(path);
                } else if value == "dump-render" {
                    let path = parser.value()?.parse()?;
                    command = ArgsCommand::DumpRender(path);
                } else {
                    return Err(arg.unexpected());
                }
            }
            Long("no-exit") => exit = false,
            Long("help") => {
                println!(
                    "Commands:\n\n\
                    dump-update-schedule <file>\n\
                    dump-render <file>\n\n\
                      --no-exit Do not exit after performing debugdump actions"
                );
                std::process::exit(0);
            }
            _ => return Err(arg.unexpected()),
        }
    }

    Ok(Args { command, exit })
}

fn execute_cli(app: &mut App) -> Result<Args, Box<dyn std::error::Error>> {
    let args = parse_args()?;

    match &args.command {
        ArgsCommand::None => Ok(args),
        ArgsCommand::DumpRender(path) => {
            let settings = render_graph::Settings::default();
            let mut out = File::create(path)?;
            write!(out, "{}", render_graph_dot(app, &settings))?;

            Ok(args)
        }
        ArgsCommand::DumpUpdateSchedule(path) => {
            let settings = schedule_graph::Settings::default();
            let mut out = File::create(path)?;
            write!(
                out,
                "{}",
                schedule_graph_dot(app, bevy_app::Update, &settings)
            )?;

            Ok(args)
        }
    }
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
