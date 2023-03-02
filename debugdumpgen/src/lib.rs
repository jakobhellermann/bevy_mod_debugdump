use std::any::TypeId;

use bevy::{prelude::*, render::RenderApp};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Context {
    app: App,
}

#[wasm_bindgen]
impl Context {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let mut app = App::default();
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some("#canvas".to_string()),
                ..default()
            }),
            ..default()
        }));

        Context { app }
    }

    pub fn generate_svg(&mut self, schedule: String, includes: String, excludes: String) -> String {
        let split = |s: &str| {
            s.split(",")
                .filter(|s| !s.is_empty())
                .map(|i| i.trim().to_owned())
                .collect::<Vec<_>>()
        };
        let includes = split(&includes);
        let excludes = split(&excludes);

        let ignore_ambiguities = &[TypeId::of::<bevy::render::texture::TextureCache>()];
        let settings = bevy_mod_debugdump::schedule_graph::Settings {
            include_system: Some(Box::new(move |system| {
                let name = system.name();
                if excludes.iter().any(|e| name.contains(e)) {
                    return false;
                }

                includes.is_empty() || includes.iter().any(|i| name.contains(i))
            })),
            ..default()
        }
        .without_single_ambiguities_on_one_of(ignore_ambiguities);

        match schedule.as_str() {
            "Main" => {
                bevy_mod_debugdump::schedule_graph_dot(&mut self.app, CoreSchedule::Main, &settings)
            }
            "Startup" => bevy_mod_debugdump::schedule_graph_dot(
                &mut self.app,
                CoreSchedule::Startup,
                &settings,
            ),
            "RenderExtract" => with_main_world_in_render_app(&mut self.app, |render_app| {
                bevy_mod_debugdump::schedule_graph_dot(render_app, ExtractSchedule, &settings)
            }),
            "RenderMain" => with_main_world_in_render_app(&mut self.app, |render_app| {
                bevy_mod_debugdump::schedule_graph_dot(render_app, CoreSchedule::Main, &settings)
            }),
            _ => panic!("unknown schedule: {schedule}"),
        }
    }
}

fn with_main_world_in_render_app<T>(app: &mut App, f: impl Fn(&mut App) -> T) -> T {
    // temporarily add the app world to the render world as a resource
    let inserted_world = std::mem::take(&mut app.world);
    let mut render_main_world = bevy::render::MainWorld::default();
    *render_main_world = inserted_world;

    let render_app = app.sub_app_mut(RenderApp);
    render_app.world.insert_resource(render_main_world);

    let ret = f(render_app);

    // move the app world back, as if nothing happened.
    let mut inserted_world = render_app
        .world
        .remove_resource::<bevy::render::MainWorld>()
        .unwrap();
    app.world = std::mem::take(&mut *inserted_world);

    ret
}
