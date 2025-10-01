use std::{fs::File, path::PathBuf};

use bevy_app::App;
use bevy_ecs::{
    intern::Interned,
    message::MessageWriter,
    schedule::{ScheduleLabel, Schedules},
};
use bevy_log::{error, info};
use std::io::Write;

#[cfg(feature = "render_graph")]
use crate::{render_graph, render_graph_dot};
use crate::{schedule_graph, schedule_graph_dot};

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
pub struct CommandLineArgs;

impl bevy_app::Plugin for CommandLineArgs {
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
        let exit = match execute_cli(app) {
            Ok(args) => args.exit,
            Err(e) => {
                error!("{e:?}");
                true
            }
        };

        if exit {
            // TODO: It would be nice if we could exit before the window
            // opens, but I don't see how.
            app.add_systems(
                bevy_app::First,
                |mut app_exit_events: MessageWriter<bevy_app::AppExit>| {
                    app_exit_events.write(bevy_app::AppExit::Success);
                },
            );
        }
    }
}

struct Args {
    command: ArgsCommand,
    exit: bool,
    /// The path to write the graph dot to. If unset, write to stdout.
    out_path: Option<PathBuf>,
}

/// A command to execute from the CLI.
enum ArgsCommand {
    None,
    /// Dumps the render graph to the specified file path.
    DumpRender,
    /// Dumps the schedule graph.
    DumpSchedule {
        /// The schedule to dump.
        schedule: String,
    },
}

fn parse_args() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;

    let mut command = ArgsCommand::None;
    let mut exit = true;
    let mut out_path = None;

    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match &arg {
            Value(value) => {
                if !matches!(command, ArgsCommand::None) {
                    return Err(arg.unexpected());
                }

                if value == "dump-schedule" {
                    let schedule = parser.value()?.parse()?;
                    command = ArgsCommand::DumpSchedule { schedule };
                } else if value == "dump-render" {
                    command = ArgsCommand::DumpRender;
                } else {
                    return Err(arg.unexpected());
                }
            }
            Short('o') | Long("output") => out_path = Some(parser.value()?.parse()?),
            Long("no-exit") => exit = false,
            Long("help") => {
                info!(
                    "Usage:\n\
                    dump-schedule <schedule_name> \n\
                    dump-render \n\n\
                      -o, --output  Write output to file instead of printing to stdout\n\
                      --no-exit     Do not exit after performing debugdump actions"
                );
                std::process::exit(0);
            }
            _ => return Err(arg.unexpected()),
        }
    }

    Ok(Args {
        command,
        exit,
        out_path,
    })
}

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

fn execute_cli(app: &mut App) -> Result<Args> {
    let mut args = parse_args()?;

    let write = |out: &str| -> Result<()> {
        match &args.out_path {
            None => {
                println!("{out}");
                Ok(())
            }
            Some(path) => {
                let mut out_file = File::create(path)?;
                write!(out_file, "{out}")?;
                Ok(())
            }
        }
    };

    match &args.command {
        ArgsCommand::None => {
            // Don't exit unless we do something here.
            args.exit = false;
            Ok(args)
        }
        #[cfg(feature = "render_graph")]
        ArgsCommand::DumpRender => {
            let settings = render_graph::Settings::default();
            write(&render_graph_dot(app, &settings))?;

            Ok(args)
        }
        #[cfg(not(feature = "render_graph"))]
        ArgsCommand::DumpRender => Err(
            "cannot dump renderer, consider enabling the feature `bevy_mod_debugdump/render_graph"
                .into(),
        ),
        ArgsCommand::DumpSchedule { schedule } => {
            let schedule = find_schedule(app, schedule)?;

            let settings = schedule_graph::Settings::default();
            write(&schedule_graph_dot(app, schedule, &settings))?;

            Ok(args)
        }
    }
}

enum FindScheduleError {
    /// There was no match. Holds the requested schedule, and the list of valid
    /// schedules by string.
    NoMatch(String, Vec<String>),
    MoreThanOneMatch(String),
}

impl std::fmt::Debug for FindScheduleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoMatch(request, schedules) => {
                f.write_fmt(format_args!("No schedules matched the requested schedule '{request}'. The valid schedules are:\n"))?;
                for schedule in schedules {
                    f.write_fmt(format_args!("\n{schedule}"))?;
                }
                Ok(())
            }
            Self::MoreThanOneMatch(request) => f.write_fmt(format_args!(
                "More than one schedule matched requested schedule '{request}'"
            )),
        }
    }
}

impl std::fmt::Display for FindScheduleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

impl std::error::Error for FindScheduleError {}

/// Looks up a schedule by its string name in `App`.
fn find_schedule(
    app: &App,
    schedule_name: &str,
) -> Result<Interned<dyn ScheduleLabel>, FindScheduleError> {
    let lower_schedule_name = schedule_name.to_lowercase();

    let schedules = app.world().resource::<Schedules>();
    let schedules = schedules
        .iter()
        // Note we get the Interned label from `schedule` since `&dyn ScheduleLabel` doesn't `impl
        // ScheduleLabel`.
        .map(|(label, schedule)| (format!("{label:?}").to_lowercase(), schedule.label()))
        .collect::<Vec<_>>();

    let mut found_label = None;
    for (str, label) in schedules.iter() {
        if str == &lower_schedule_name {
            if found_label.is_some() {
                return Err(FindScheduleError::MoreThanOneMatch(
                    schedule_name.to_string(),
                ));
            }
            found_label = Some(*label);
        }
    }

    if let Some(label) = found_label {
        Ok(label)
    } else {
        Err(FindScheduleError::NoMatch(
            schedule_name.to_string(),
            schedules.into_iter().map(|(str, _)| str).collect(),
        ))
    }
}
