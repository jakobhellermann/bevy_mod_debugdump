#![allow(clippy::needless_doctest_main)]
use bevy::{prelude::*, render2::RenderApp};

mod dot;

#[cfg(feature = "render_graph")]
pub mod render_graph;
pub mod schedule_graph;

/// Prints the current render graph using [render_graph_dot](render_graph::render_graph_dot).
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
    use bevy::render2::render_graph::RenderGraph;

    let render_app = app.get_sub_app(RenderApp).expect("no render app");
    let render_graph = render_app.world.get_resource::<RenderGraph>().unwrap();

    let dot = render_graph::render_graph_dot(&*render_graph);
    println!("{}", dot);
}

/// Prints the system schedule of the render sub-app.
/// ```rust,no_run
/// use bevy::prelude::*;
///
/// fn main() {
///     let mut app = App::new();
///     app.add_plugins(DefaultPlugins);
///     bevy_mod_debugdump::print_render_schedule_graph(&mut app);
/// }
pub fn print_render_schedule_graph(app: &mut App) {
    let render_app = app.get_sub_app(RenderApp).expect("no render app");

    let default_style = schedule_graph::ScheduleGraphStyle {
        hide_startup_schedule: false,
        ..schedule_graph::ScheduleGraphStyle::dark()
    };
    println!(
        "{}",
        schedule_graph::schedule_graph_dot_styled(&render_app.schedule, &default_style)
    );
}

/// Prints the app system schedule using [schedule_graph_dot](schedule_graph::schedule_graph_dot) as a dot graph and exits.
/// # Example
/// ```rust,no_run
/// use bevy::prelude::*;
///
/// fn main() {
///     let mut app = App::new();
///     app.add_plugins(DefaultPlugins);
///     bevy_mod_debugdump::print_schedule(&mut app);
/// }
/// ```
pub fn print_schedule(app: &mut App) {
    println!("{}", schedule_graph::schedule_graph_dot(&app.schedule));
}
