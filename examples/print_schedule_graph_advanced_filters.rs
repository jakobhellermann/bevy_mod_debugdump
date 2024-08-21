use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_app::DynEq;
use bevy_mod_debugdump::schedule_graph::Settings;

fn special_system_a() {}
fn special_system_b() {}
fn regular_system_c() {}

#[derive(Debug, Clone, Hash, PartialEq, Eq, SystemSet)]
enum ExampleSystemSet {
    Special,
    Regular,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>())
        .configure_sets(
            Update,
            (ExampleSystemSet::Special, ExampleSystemSet::Regular),
        )
        .add_systems(
            Update,
            (special_system_a, special_system_b, regular_system_c).chain(),
        );

    let settings = Settings::default()
        .filter_in_crate("print_schedule_graph_advanced_filters")
        .with_system_filter(|system| system.name().contains("special"))
        .with_system_set_filter(|system_set| {
            system_set
                .as_dyn_eq()
                .dyn_eq(ExampleSystemSet::Special.as_dyn_eq())
        });
    let dot = bevy_mod_debugdump::schedule_graph_dot(&mut app, Update, &settings);
    println!("{dot}");
}
