use std::path::PathBuf;

use bevy::prelude::*;
use bevy_mod_debugdump_stageless::Settings;

fn main() -> Result<(), std::io::Error> {
    let docs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs");
    std::fs::create_dir_all(&docs_path)?;

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

            let main = schedules.get(&CoreSchedule::Main).unwrap();
            let main_filtered_settings = Settings {
                show_ambiguities: false,
                include_system: Box::new(|system| {
                    let name = system.name();
                    name.contains("buffers")
                }),
                ..settings
            };
            let dot = bevy_mod_debugdump_stageless::schedule_to_dot(
                main,
                &world,
                &main_filtered_settings,
            );

            std::fs::write(docs_path.join(format!("schedule_Main_filtered.dot")), dot)?;

            Ok(())
        })
}
