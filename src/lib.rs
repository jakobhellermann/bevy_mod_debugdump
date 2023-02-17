use bevy_app::{App, CoreSchedule};
use bevy_ecs::schedule::{ScheduleLabel, Schedules};

mod dot;

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
    let dot = schedule_graph_dot(
        app,
        CoreSchedule::Main,
        &schedule_graph::Settings::default(),
    );
    println!("{dot}");
}
