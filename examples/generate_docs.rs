use std::{any::TypeId, path::PathBuf};

use bevy::{
    core_pipeline::core_3d::Transmissive3d,
    pbr::{MeshInputUniform, MeshUniform},
    prelude::*,
    render::RenderApp,
};
use bevy_mod_debugdump::schedule_graph::{settings::Style, Settings};
use bevy_render::{
    batching::{
        gpu_preprocessing::{BatchedInstanceBuffers, IndirectParametersBuffers},
        no_gpu_preprocessing::BatchedInstanceBuffer,
    },
    render_asset::RenderAssetBytesPerFrame,
    render_phase::ViewSortedRenderPhases,
};

fn main() -> Result<(), std::io::Error> {
    let docs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs");
    let schedule_path = docs_path.join("schedule");
    let render_path = docs_path.join("render");
    std::fs::create_dir_all(schedule_path.join("light"))?;
    std::fs::create_dir_all(schedule_path.join("dark"))?;
    std::fs::create_dir_all(render_path.join("light"))?;
    std::fs::create_dir_all(render_path.join("dark"))?;

    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    let style_light = Style::light();
    let style_dark = Style::dark_github();

    app.world_mut()
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
                    world,
                    &settings_light,
                );
                let dot_dark = bevy_mod_debugdump::schedule_graph::schedule_graph_dot(
                    schedule,
                    world,
                    &settings_dark,
                );

                let filename = format!("schedule_{label:?}.dot");
                std::fs::write(schedule_path.join("light").join(&filename), dot_light)?;
                std::fs::write(schedule_path.join("dark").join(&filename), dot_dark)?;
            }

            Ok::<_, std::io::Error>(())
        })?;

    with_main_world_in_render_app(&mut app, |render_app| {
        render_app
            .world_mut()
            .resource_scope::<Schedules, _>(|world, mut schedules| {
                let ignored_ambiguities = schedules.ignored_scheduling_ambiguities.clone();

                for (label, schedule) in schedules.iter_mut() {
                    // for access info
                    schedule.graph_mut().initialize(world);
                    // for `conflicting_systems`

                    schedule
                        .graph_mut()
                        .build_schedule(world, &ignored_ambiguities)
                        .unwrap();

                    let ignore_ambiguities = &[
                        TypeId::of::<bevy_render::MainWorld>(),
                        TypeId::of::<bevy_render::texture::TextureCache>(),
                        TypeId::of::<IndirectParametersBuffers>(),
                        TypeId::of::<BatchedInstanceBuffers<MeshUniform, MeshInputUniform>>(),
                        TypeId::of::<BatchedInstanceBuffer<MeshUniform>>(),
                        TypeId::of::<RenderAssetBytesPerFrame>(),
                        TypeId::of::<ViewSortedRenderPhases<Transmissive3d>>(),
                    ];
                    let settings_light = Settings {
                        style: style_light.clone(),
                        ..Default::default()
                    }
                    .without_single_ambiguities_on_one_of(ignore_ambiguities);
                    let settings_dark = Settings {
                        style: style_dark.clone(),
                        ..Default::default()
                    }
                    .without_single_ambiguities_on_one_of(ignore_ambiguities);

                    let dot_light = bevy_mod_debugdump::schedule_graph::schedule_graph_dot(
                        schedule,
                        world,
                        &settings_light,
                    );
                    let dot_dark = bevy_mod_debugdump::schedule_graph::schedule_graph_dot(
                        schedule,
                        world,
                        &settings_dark,
                    );

                    let filename = format!("render_schedule_{label:?}.dot");
                    std::fs::write(schedule_path.join("light").join(&filename), dot_light)?;
                    std::fs::write(schedule_path.join("dark").join(&filename), dot_dark)?;
                }
                Ok::<(), std::io::Error>(())
            })
    })?;

    let settings_render_light = bevy_mod_debugdump::render_graph::Settings {
        style: bevy_mod_debugdump::render_graph::settings::Style::light(),
    };
    let settings_render_dark = bevy_mod_debugdump::render_graph::Settings {
        style: bevy_mod_debugdump::render_graph::settings::Style::dark_github(),
    };
    let dot_light = bevy_mod_debugdump::render_graph_dot(&app, &settings_render_light);
    let dot_dark = bevy_mod_debugdump::render_graph_dot(&app, &settings_render_dark);
    let filename = "render_graph.dot";
    std::fs::write(render_path.join("light").join(filename), dot_light)?;
    std::fs::write(render_path.join("dark").join(filename), dot_dark)?;

    Ok(())
}

fn initialize_schedules(
    schedules: &mut Mut<Schedules>,
    world: &mut World,
) -> Result<(), std::io::Error> {
    let ignored_ambiguities = schedules.ignored_scheduling_ambiguities.clone();
    for (_, schedule) in schedules.iter_mut() {
        // for access info
        schedule.graph_mut().initialize(world);
        // for `conflicting_systems`
        schedule
            .graph_mut()
            .build_schedule(world, &ignored_ambiguities)
            .unwrap();
    }
    Ok(())
}

fn with_main_world_in_render_app<T>(app: &mut App, f: impl Fn(&mut SubApp) -> T) -> T {
    // temporarily add the app world to the render world as a resource
    let inserted_world = std::mem::take(app.world_mut());
    let mut render_main_world = bevy_render::MainWorld::default();
    *render_main_world = inserted_world;

    let render_app = app.sub_app_mut(RenderApp);
    render_app.world_mut().insert_resource(render_main_world);

    let ret = f(render_app);

    // move the app world back, as if nothing happened.
    let mut inserted_world = render_app
        .world_mut()
        .remove_resource::<bevy_render::MainWorld>()
        .unwrap();
    *app.world_mut() = std::mem::take(&mut *inserted_world);

    ret
}
