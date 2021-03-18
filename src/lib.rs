use bevy::prelude::*;

mod dot;

#[cfg(feature = "render_graph")]
mod render_graph;
mod schedule_graph;
#[cfg(feature = "render_graph")]
pub use render_graph::render_graph_dot;
pub use schedule_graph::schedule_graph_dot;

/// System which prints the current render graph.
/// # Example
/// ```rust,no_run
/// use bevy::prelude::*;
///
/// fn main() {
///     App::build()
///         // .insert_resource(Msaa { samples: 4 })
///         .add_plugins(DefaultPlugins)
///         .add_startup_system(bevy_mod_debugdump::print_render_graph.system())
///         .run();
/// }
/// ```
#[cfg(feature = "render_graph")]
pub fn print_render_graph(render_graph: Res<bevy::render::render_graph::RenderGraph>) {
    let dot = render_graph_dot(&*render_graph);
    println!("{}", dot);
}

/// App runner which prints the schedule as a dot graph and exits.
/// # Example
/// ```rust,no_run
/// use bevy::prelude::*;
///
/// fn main() {
///     let mut app = App::build();
///     app.add_plugins(DefaultPlugins);
///     app.set_runner(bevy_mod_debugdump::print_schedule_runner);
///     app.run();
/// }
/// ```
pub fn print_schedule_runner(app: App) {
    /*
    use bevy::app::Events;
    use bevy::window::{WindowCreated, WindowId};
    use bevy::winit::WinitWindows;

    {
        let world_cell = app.world.cell();
        let mut window_created_events = world_cell
            .get_resource_mut::<Events<WindowCreated>>()
            .unwrap();
        let mut windows = world_cell.get_resource_mut::<Windows>().unwrap();
        let mut winit_windows = world_cell.get_resource_mut::<WinitWindows>().unwrap();

        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit_windows.create_window(
            &*event_loop,
            WindowId::primary(),
            &WindowDescriptor::default(),
        );
        windows.add(window);
        window_created_events.send(WindowCreated {
            id: WindowId::primary(),
        });
    }

    app.update();*/

    println!("{}", schedule_graph_dot(&app.schedule));
}
