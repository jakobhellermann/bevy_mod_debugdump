use std::path::PathBuf;

use bevy::{prelude::*, render::RenderApp, utils::HashSet};
use bevy_mod_debugdump::schedule_graph::{settings::Style, Settings};

fn main() -> Result<(), std::io::Error> {
    let docs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs");
    let schedule_path = docs_path.join("schedule");
    let schedule_path_by_crate = schedule_path.join("by-crate");
    std::fs::create_dir_all(schedule_path.join("light"))?;
    std::fs::create_dir_all(schedule_path.join("dark"))?;
    std::fs::create_dir_all(&schedule_path_by_crate.join("light"))?;
    std::fs::create_dir_all(&schedule_path_by_crate.join("dark"))?;

    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    let style_light = Style::light();
    let style_dark = Style::dark_github();

    app.world
        .resource_scope::<Schedules, _>(|world, mut schedules| {
            initialize_schedules(&mut schedules, world)?;

            let settings_light = Settings {
                style: style_light.clone(),
                ..Settings::default()
            };
            let settings_dark = Settings {
                style: style_dark.clone(),
                ..Settings::default()
            };
            for (label, schedule) in schedules.iter() {
                let dot_light = bevy_mod_debugdump::schedule_graph::schedule_graph_dot(
                    schedule,
                    &world,
                    &settings_light,
                );
                let dot_dark = bevy_mod_debugdump::schedule_graph::schedule_graph_dot(
                    schedule,
                    &world,
                    &settings_dark,
                );

                let filename = format!("schedule_{label:?}.dot");
                std::fs::write(schedule_path.join("light").join(&filename), dot_light)?;
                std::fs::write(schedule_path.join("dark").join(&filename), dot_dark)?;
            }

            // filtered main, without mass event/asset systems
            let main = schedules.get(&CoreSchedule::Main).unwrap();

            let filter = |system: &dyn System<In = (), Out = ()>| {
                let name = system.name();
                let ignore = ["asset_event_system", "update_asset_storage_system"];
                let events_update_system = name.starts_with("bevy_ecs::event::Events")
                    && name.ends_with("::update_system");
                !events_update_system && !ignore.iter().any(|remove| name.contains(remove))
            };
            let main_filtered_settings_light = Settings {
                include_system: Some(Box::new(filter)),
                style: style_light.clone(),
                ..Settings::default()
            };
            let main_filtered_settings_dark = Settings {
                include_system: Some(Box::new(filter)),
                style: style_dark.clone(),
                ..Settings::default()
            };
            let dot_light = bevy_mod_debugdump::schedule_graph::schedule_graph_dot(
                main,
                &world,
                &main_filtered_settings_light,
            );
            let dot_dark = bevy_mod_debugdump::schedule_graph::schedule_graph_dot(
                main,
                &world,
                &main_filtered_settings_dark,
            );

            let filename = format!("schedule_Main_Filtered.dot");
            std::fs::write(schedule_path.join("light").join(&filename), dot_light)?;
            std::fs::write(schedule_path.join("dark").join(&filename), dot_dark)?;

            // by crate
            let bevy_crates: HashSet<_> = main
                .graph()
                .systems()
                .filter_map(|(_, system, _, _)| Some(system.name().split_once("::")?.0.to_owned()))
                .collect();

            for bevy_crate in bevy_crates {
                let bevy_crate_clone = bevy_crate.clone();
                let by_crate_settings_light = Settings {
                    include_system: Some(Box::new(move |system| {
                        let bevy_crate = bevy_crate_clone.clone();
                        let name = system.name();
                        name.starts_with(&bevy_crate)
                    })),
                    style: style_light.clone(),
                    ..Default::default()
                };
                let bevy_crate_clone = bevy_crate.clone();
                let by_crate_settings_dark = Settings {
                    include_system: Some(Box::new(move |system| {
                        let bevy_crate = bevy_crate_clone.clone();
                        let name = system.name();
                        name.starts_with(&bevy_crate)
                    })),
                    style: style_dark.clone(),
                    ..Default::default()
                };

                let dot_light = bevy_mod_debugdump::schedule_graph::schedule_graph_dot(
                    main,
                    &world,
                    &by_crate_settings_light,
                );
                let dot_dark = bevy_mod_debugdump::schedule_graph::schedule_graph_dot(
                    main,
                    &world,
                    &by_crate_settings_dark,
                );

                let filename = format!("schedule_Main_{}.dot", bevy_crate);
                std::fs::write(
                    schedule_path_by_crate.join("light").join(&filename),
                    dot_light,
                )?;
                std::fs::write(
                    schedule_path_by_crate.join("dark").join(&filename),
                    dot_dark,
                )?;
            }

            Ok::<_, std::io::Error>(())
        })?;

    let render_app = app.sub_app_mut(RenderApp);
    render_app
        .world
        .resource_scope::<Schedules, _>(|world, mut schedules| {
            for (label, schedule) in schedules.iter_mut() {
                // TODO: currently panics
                // for access info
                // schedule.graph_mut().initialize(world);
                // for `conflicting_systems`
                schedule
                    .graph_mut()
                    .build_schedule(world.components())
                    .unwrap();

                let settings_light = Settings {
                    style: style_light.clone(),
                    ..Default::default()
                };
                let settings_dark = Settings {
                    style: style_dark.clone(),
                    ..Default::default()
                };

                let dot_light = bevy_mod_debugdump::schedule_graph::schedule_graph_dot(
                    schedule,
                    &world,
                    &settings_light,
                );
                let dot_dark = bevy_mod_debugdump::schedule_graph::schedule_graph_dot(
                    schedule,
                    &world,
                    &settings_dark,
                );

                let filename = format!("render_schedule_{label:?}.dot");
                std::fs::write(schedule_path.join("light").join(&filename), dot_light)?;
                std::fs::write(schedule_path.join("dark").join(&filename), dot_dark)?;
            }
            Ok::<(), std::io::Error>(())
        })?;

    Ok(())
}

fn initialize_schedules(
    schedules: &mut Mut<Schedules>,
    world: &mut World,
) -> Result<(), std::io::Error> {
    Ok(for (_, schedule) in schedules.iter_mut() {
        // for access info
        schedule.graph_mut().initialize(world);
        // for `conflicting_systems`
        schedule
            .graph_mut()
            .build_schedule(world.components())
            .unwrap();
    })
}
