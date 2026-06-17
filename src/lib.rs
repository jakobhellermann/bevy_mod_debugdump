#![allow(clippy::needless_doctest_main)]
#![allow(clippy::type_complexity)]

use bevy_app::App;
use bevy_ecs::schedule::{ScheduleLabel, Schedules};

#[cfg(feature = "cli")]
mod cli;
mod dot;

pub mod schedule_graph;

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

            let label_name = format!("{label:?}");
            let schedule = schedules
                .get_mut(label)
                .ok_or_else(|| format!("schedule {label_name} doesn't exist"))
                .unwrap();
            schedule.graph_mut().initialize(world);
            let _ = schedule
                .graph_mut()
                .build_schedule(world, &ignored_ambiguities);

            schedule_graph::schedule_graph_dot(schedule, world, settings)
        })
}

/// Prints the schedule with default settings.
pub fn print_schedule_graph(app: &mut App, schedule_label: impl ScheduleLabel) {
    let dot = schedule_graph_dot(app, schedule_label, &schedule_graph::Settings::default());
    println!("{dot}");
}

#[cfg(feature = "cli")]
pub use cli::CommandLineArgs;
