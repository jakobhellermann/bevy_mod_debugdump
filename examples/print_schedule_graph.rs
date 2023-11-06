use bevy::log::LogPlugin;
use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>());
    bevy_mod_debugdump::print_schedule_graph(&mut app, PostUpdate);
}
