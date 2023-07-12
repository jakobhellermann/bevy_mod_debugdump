use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_mod_debugdump::schedule_graph::Settings;

fn system_a() {}
fn system_b() {}
fn system_c() {}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>());
    app.add_systems(Update, (system_a, system_b, system_c).chain());

    let settings = Settings::default().filter_in_crate("print_schedule_graph_filtered");
    let dot = bevy_mod_debugdump::schedule_graph_dot(&mut app, Update, &settings);
    println!("{dot}");
}
