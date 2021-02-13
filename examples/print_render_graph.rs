use bevy::{prelude::*, render::render_graph::RenderGraph};

use bevy_mod_debugdump::render_graph_dot;

fn main() {
    App::build()
        // .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_startup_system(debug.system())
        .run();
}
fn debug(render_graph: Res<RenderGraph>) {
    let dot = render_graph_dot(&*render_graph);
    println!("{}", dot);
    std::process::exit(0);
}
