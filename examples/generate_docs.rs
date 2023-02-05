use std::path::PathBuf;

use bevy::{prelude::*, utils::HashSet};
use bevy_mod_debugdump_stageless::Settings;

fn main() -> Result<(), std::io::Error> {
    let docs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs");
    let docs_path_by_crate = docs_path.join("by-crate");
    std::fs::create_dir_all(&docs_path)?;
    std::fs::create_dir_all(&docs_path_by_crate)?;

    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    let settings = Settings {
        ..Default::default()
    };

    app.world
        .resource_scope::<Schedules, _>(|world, mut schedules| {
            for (label, schedule) in schedules.iter_mut() {
                // for access info
                schedule.graph_mut().initialize(world);
                // for `conflicting_systems`
                schedule
                    .graph_mut()
                    .build_schedule(world.components())
                    .unwrap();

                let dot =
                    bevy_mod_debugdump_stageless::schedule_to_dot(schedule, &world, &settings);

                std::fs::write(docs_path.join(format!("schedule_{label:?}.dot")), dot)?;
            }

            // filtered main, without mass event/asset systems
            let main = schedules.get(&CoreSchedule::Main).unwrap();
            let main_filtered_settings = Settings {
                ambiguity_enable: false,
                include_system: Box::new(|system| {
                    let name = system.name();
                    let ignore = ["asset_event_system", "update_asset_storage_system"];
                    let events_update_system = name.starts_with("bevy_ecs::event::Events")
                        && name.ends_with("::update_system");
                    !events_update_system && !ignore.iter().any(|remove| name.contains(remove))
                }),
                ..settings
            };
            let dot = bevy_mod_debugdump_stageless::schedule_to_dot(
                main,
                &world,
                &main_filtered_settings,
            );

            std::fs::write(docs_path.join(format!("schedule_Main_filtered.dot")), dot)?;

            // by crate
            let bevy_crates: HashSet<_> = main
                .graph()
                .systems()
                .filter_map(|(_, system, _)| Some(system.name().split_once("::")?.0.to_owned()))
                .collect();

            for bevy_crate in bevy_crates {
                let bevy_crate_clone = bevy_crate.clone();
                let by_crate_settings = Settings {
                    include_system: Box::new(move |system| {
                        let bevy_crate = bevy_crate_clone.clone();
                        let name = system.name();
                        name.starts_with(&bevy_crate)
                    }),
                    ..Default::default()
                };

                let dot =
                    bevy_mod_debugdump_stageless::schedule_to_dot(main, &world, &by_crate_settings);
                std::fs::write(
                    docs_path_by_crate.join(format!("schedule_Main_{}.dot", bevy_crate)),
                    dot,
                )?;
            }

            Ok(())
        })
}
