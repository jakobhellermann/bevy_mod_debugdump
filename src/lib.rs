#![allow(clippy::needless_doctest_main)]

mod dot;

#[cfg(feature = "render_graph")]
pub mod render_graph;
pub mod schedule_graph;

use bevy_app::App;
#[cfg(feature = "bevy_render")]
use bevy_render::{RenderApp, RenderStage};

/// Prints the main system schedule using [`schedule_graph_dot`](schedule_graph::schedule_graph_dot) as a dot graph.
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
    app.update();
    println!("{}", schedule_graph::schedule_graph_dot(app));
}

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
    use bevy_render::render_graph::RenderGraph;

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
///     bevy_mod_debugdump::print_render_schedule(&mut app);
/// }
#[cfg(feature = "render_graph")]
pub fn print_render_schedule(app: &mut App) {
    app.update();

    let dot = schedule_graph::schedule_graph_dot_sub_app_styled(
        app,
        RenderApp,
        &[&RenderStage::Extract],
        &schedule_graph::ScheduleGraphStyle::default(),
    );
    println!("{}", dot);
}
