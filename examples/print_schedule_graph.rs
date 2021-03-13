use bevy::prelude::*;
use bevy_mod_debugdump::schedule_graph_dot;

fn main() {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins);

    app.set_runner(|mut app| {
        app.update();
        println!("{}", schedule_graph_dot(&app.schedule));
    });
}
