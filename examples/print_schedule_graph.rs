use bevy::log::LogPlugin;
use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>());
    app.world_mut().send_event(AppExit::Success);
    app.add_systems(Update, |world: &mut World| {
        // dbg!();
        bevy_mod_debugdump::print_schedule_graph(world, PostUpdate);
    });

    app.run();
}
