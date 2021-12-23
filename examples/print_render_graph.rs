use bevy::{log::LogPlugin, prelude::*};

fn main() {
    let mut app = App::new();
    // .insert_resource(Msaa { samples: 4 })
    app.add_plugins_with(DefaultPlugins, |plugins| {
        plugins.disable::<LogPlugin>()
    });

    bevy_mod_debugdump::print_render_graph(&mut app);
}
