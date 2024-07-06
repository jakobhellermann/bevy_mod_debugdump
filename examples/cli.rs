use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy_mod_debugdump::CommandLineArgs)
        .run();
}
