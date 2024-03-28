use bevy::log::LogPlugin;
use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>());
    bevy_mod_debugdump::print_events_graph(
        &mut app,
        vec![
            Box::new(First),
            Box::new(PreUpdate),
            Box::new(Update),
            Box::new(PostUpdate),
        ],
    );
}
