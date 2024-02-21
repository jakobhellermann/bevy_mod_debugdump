use std::any::TypeId;

use bevy::{app::MainScheduleOrder, ecs::schedule::ScheduleLabel, prelude::*, render::RenderApp};
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

        #[derive(States, PartialEq, Eq, Clone, Debug, Hash, Default)]
        enum ExampleState1 {
            #[default]
            A,
        }
        #[derive(States, PartialEq, Eq, Clone, Debug, Hash, Default)]
        enum ExampleState2 {
            #[default]
            A,
        }
        app.init_state::<ExampleState1>();
        app.init_state::<ExampleState2>();

        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some("#canvas".to_string()),
                ..default()
            }),
            ..default()
        }));

        Context { app }
    }

    pub fn main_schedules(&self) -> Vec<JsValue> {
        let main_schedule_order = self.app.world.resource::<MainScheduleOrder>();
        main_schedule_order
            .labels
            .iter()
            .map(|label| JsValue::from(format!("{:?}", *label)))
            .collect()
    }
    pub fn non_main_schedules(&self) -> Vec<JsValue> {
        let main_schedule_order = self.app.world.resource::<MainScheduleOrder>();
        let schedules = self.app.world.resource::<Schedules>();

        schedules
            .iter()
            .filter_map(|(label, _)| {
                let in_main = main_schedule_order
                    .labels
                    .iter()
                    .any(|main| **main == *label);

                if !in_main {
                    Some(JsValue::from(format!("{label:?}")))
                } else {
                    None
                }
            })
            .collect()
    }
    pub fn render_schedules(&self) -> Vec<JsValue> {
        let schedules = self.app.sub_app(RenderApp).world.resource::<Schedules>();

        schedules
            .iter()
            .map(|(label, _)| JsValue::from(format!("{label:?}")))
            .collect()
    }

    pub fn generate_svg(
        &mut self,
        schedule_label: String,
        render_app: bool,
        includes: String,
        excludes: String,
    ) -> Result<String, String> {
        choose_app(&mut self.app, render_app, |app| {
            app.world
                .resource_scope::<Schedules, _>(|world, mut schedules| {
                    let ignored_ambiguities = schedules.ignored_scheduling_ambiguities.clone();

                    let (_, schedule) = schedules
                        .iter_mut()
                        .find_map(|(label, schedule)| {
                            (format!("{:?}", label) == schedule_label).then_some((label, schedule))
                        })
                        .ok_or_else(|| {
                            format!(
                                "schedule '{schedule_label}' not found in {}app",
                                if render_app { "render " } else { "" }
                            )
                        })?;

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

                    let settings = &settings;
                    schedule.graph_mut().initialize(world);
                    let _ = schedule.graph_mut().build_schedule(
                        world.components(),
                        ScheduleDebugGroup.intern(),
                        &ignored_ambiguities,
                    );

                    Ok(bevy_mod_debugdump::schedule_graph::schedule_graph_dot(
                        schedule, world, settings,
                    ))
                })
        })
    }
}

fn choose_app<T>(main_app: &mut App, render_app: bool, f: impl Fn(&mut App) -> T) -> T {
    if render_app {
        with_main_world_in_render_app(main_app, f)
    } else {
        f(main_app)
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

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct ScheduleDebugGroup;
