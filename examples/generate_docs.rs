use std::path::PathBuf;

use bevy::prelude::*;

fn main() -> Result<(), std::io::Error> {
    let docs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs");
    std::fs::create_dir_all(&docs_path)?;

    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

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

                let settings = bevy_mod_debugdump_stageless::Settings::default();
                let dot =
                    bevy_mod_debugdump_stageless::schedule_to_dot(schedule, &world, &settings);

                std::fs::write(docs_path.join(format!("schedule_{label:?}.dot")), dot)?;
            }

            Ok(())
        })
}
