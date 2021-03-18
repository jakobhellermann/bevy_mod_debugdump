use bevy::prelude::*;

fn main() {
    App::build()
        // .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_startup_system(bevy_mod_debugdump::print_render_graph.system())
        .run();
}
