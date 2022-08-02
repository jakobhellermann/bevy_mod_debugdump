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
    println!("{}", get_schedule(app));
}

/// Returns the main system schedule using [`schedule_graph_dot`](schedule_graph::schedule_graph_dot) as a dot graph.
/// # Example
/// ```rust,no_run
/// use bevy::prelude::*;
/// use std::fs::File;
/// use std::io::Write;
///
/// fn main() {
///     let mut app = App::new();
///     app.add_plugins(DefaultPlugins);
///     let system_schedule = bevy_mod_debugdump::get_schedule(&mut app);
///     let mut file = File::create("system_schedule.dot").unwrap();
///     file.write_all(system_schedule.as_bytes()).unwrap();
/// }
/// ```
pub fn get_schedule(app: &mut App) -> String {
    app.update();
    schedule_graph::schedule_graph_dot(app)
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
    println!("{}", get_render_graph(app));
}

/// Returns the current render graph using [render_graph_dot](render_graph::render_graph_dot).
/// # Example
/// ```rust,no_run
/// use bevy::prelude::*;
/// use std::fs::File;
/// use std::io::Write;
///
/// fn main() {
///     let mut app = App::new();
///     app.add_plugins(DefaultPlugins);
///     let render_graph = bevy_mod_debugdump::get_render_graph(&mut app);
///     let mut file = File::create("render_graph.dot").unwrap();
///     file.write_all(render_graph.as_bytes()).unwrap();
/// }
/// ```
#[cfg(feature = "render_graph")]
pub fn get_render_graph(app: &mut App) -> String {
    use bevy_render::render_graph::RenderGraph;

    let render_app = app
        .get_sub_app(RenderApp)
        .unwrap_or_else(|_| panic!("no render app"));
    let render_graph = render_app.world.get_resource::<RenderGraph>().unwrap();

    render_graph::render_graph_dot(&*render_graph)
}

/// Prints the system schedule of the render sub-app.
/// # Example
/// ```rust,no_run
/// use bevy::prelude::*;
///
/// fn main() {
///     let mut app = App::new();
///     app.add_plugins(DefaultPlugins);
///     bevy_mod_debugdump::print_render_schedule(&mut app);
/// }
/// ```
#[cfg(feature = "render_graph")]
pub fn print_render_schedule(app: &mut App) {
    println!("{}", get_render_schedule(app));
}

/// Returns the system schedule of the render sub-app.
/// # Example
/// ```rust,no_run
/// use bevy::prelude::*;
/// use std::fs::File;
/// use std::io::Write;
///
/// fn main() {
///     let mut app = App::new();
///     app.add_plugins(DefaultPlugins);
///     let render_schedule = bevy_mod_debugdump::get_render_schedule(&mut app);
///     let mut file = File::create("render_schedule.dot").unwrap();
///     file.write_all(render_schedule.as_bytes()).unwrap();
/// }
/// ```
#[cfg(feature = "render_graph")]
pub fn get_render_schedule(app: &mut App) -> String {
    app.update();

    schedule_graph::schedule_graph_dot_sub_app_styled(
        app,
        RenderApp,
        &[&RenderStage::Extract],
        &schedule_graph::ScheduleGraphStyle::default(),
    )
}
