use bevy::log::LogPlugin;
use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins_with(DefaultPlugins, |plugins| plugins.disable::<LogPlugin>());

    bevy_mod_debugdump::print_render_graph(&mut app);
}
