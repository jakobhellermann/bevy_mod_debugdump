use std::sync::LazyLock;

use bevy_color::{Color, Srgba};
use bevy_ecs::system::ScheduleSystem;
use bevy_platform::collections::hash_map::HashMap;

static CRATE_COLORS: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    [
        // Beige/Red
        ("bevy_transform", "FFE7B9"),
        ("bevy_animation", "FFBDB9"),
        // Greys
        ("bevy_asset", "D1CBC5"),
        ("bevy_scene", "BACFCB"),
        ("bevy_time", "C7DDBD"),
        // Greens
        ("bevy_core", "3E583C"),
        ("bevy_app", "639D18"),
        ("bevy_ecs", "B0D34A"),
        ("bevy_hierarchy", "E4FBA3"),
        // Turquesa
        ("bevy_audio", "98F1D1"),
        // Purples/Pinks
        ("bevy_winit", "664F72"),
        ("bevy_a11y", "9163A6"),
        ("bevy_window", "BB85D4"),
        ("bevy_text", "E9BBFF"),
        ("bevy_gilrs", "973977"),
        ("bevy_input", "D36AAF"),
        ("bevy_ui", "FFB1E5"),
        // Blues
        ("bevy_render", "70B9FC"),
        ("bevy_pbr", "ABD5FC"),
    ]
    .into_iter()
    .collect()
});

pub struct SystemStyle {
    pub bg_color: Color,
    pub text_color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: f32,
}

pub fn color_to_hex(color: Color) -> String {
    format!(
        "#{:0>2x}{:0>2x}{:0>2x}",
        (color.to_srgba().red * 255.0) as u8,
        (color.to_srgba().green * 255.0) as u8,
        (color.to_srgba().blue * 255.0) as u8,
    )
}

pub fn system_to_style(system: &ScheduleSystem) -> SystemStyle {
    let name = system.name();
    let pretty_name = disqualified::ShortName(&name).to_string();
    let is_apply_system_buffers = pretty_name == "apply_system_buffers";
    let name_without_event = name
        .trim_start_matches("bevy_ecs::event::Events<")
        .trim_end_matches(">::update_system");
    let crate_name = name_without_event.split("::").next();

    if is_apply_system_buffers {
        SystemStyle {
            bg_color: Srgba::hex("E70000").unwrap().into(),
            text_color: Some(Srgba::hex("ffffff").unwrap().into()),
            border_color: Some(Srgba::hex("5A0000").unwrap().into()),
            border_width: 2.0,
        }
    } else {
        let bg_color = crate_name
            .and_then(|n| CRATE_COLORS.get(n))
            .map(Srgba::hex)
            .unwrap_or(Srgba::hex("eff1f3"))
            .unwrap()
            .into();

        SystemStyle {
            bg_color,
            text_color: None,
            border_color: None,
            border_width: 1.0,
        }
    }
}
