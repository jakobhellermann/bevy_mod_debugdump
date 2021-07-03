use bevy::{log::LogPlugin, PipelinedDefaultPlugins, prelude::*};

fn main() {
    let mut app = App::new();
    app.add_plugins_with(PipelinedDefaultPlugins, |plugins| plugins.disable::<LogPlugin>());
    bevy_mod_debugdump::print_schedule(&mut app);
}
