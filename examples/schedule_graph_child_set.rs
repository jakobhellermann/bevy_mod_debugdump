use bevy::{log::LogPlugin, prelude::*};

/// A set for rapier's copying bevy_rapier's Bevy components back into rapier.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct SystemSet1;

/// A set for rapier's copying bevy_rapier's Bevy components back into rapier.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct SystemSet2;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>());
    app.add_systems(
        Update,
        (
            system1_no_set,
            (
                (
                    system_in_child_set1.in_set(SystemSet1),
                    system_in_child_set1.in_set(SystemSet1),
                ),
                system_in_child_set2.in_set(SystemSet2),
            )
                .chain(),
        )
            .chain(),
    );
    bevy_mod_debugdump::print_schedule_graph(&mut app, Update);
}

fn system_in_child_set1() {}

fn system_in_child_set2() {}
fn system1_no_set() {}
