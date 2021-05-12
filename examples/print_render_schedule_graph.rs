use bevy::{prelude::*, render::render_graph::RenderGraph};
use bevy_mod_debugdump::schedule_graph::schedule_graph_dot;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(print_render_schedule.system())
        .run();
}

pub fn print_render_schedule(mut render_graph: ResMut<RenderGraph>) {
    let schedule = render_graph.take_schedule().unwrap();
    let dot = schedule_graph_dot(&schedule);
    render_graph.set_schedule(schedule);
    println!("{}", dot);
}
