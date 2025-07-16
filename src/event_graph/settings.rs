use bevy_app::{First, PostUpdate, PreUpdate, Update};
use bevy_ecs::{
    component::ComponentInfo,
    schedule::{Schedule, ScheduleLabel},
    system::System,
};
use bevy_render::color::Color;

use super::system_style::{
    color_to_hex, event_to_style, system_to_style, ComponentInfoStyle, SystemStyle,
};

#[derive(Default, Clone, Copy)]
pub enum RankDir {
    TopDown,
    #[default]
    LeftRight,
}
impl RankDir {
    pub(crate) fn as_dot(&self) -> &'static str {
        match self {
            RankDir::TopDown => "TD",
            RankDir::LeftRight => "LR",
        }
    }
}

#[derive(Default, Clone, Copy)]
pub enum EdgeStyle {
    None,
    Line,
    Polyline,
    Curved,
    Ortho,
    #[default]
    Spline,
}
impl EdgeStyle {
    pub fn as_dot(&self) -> &'static str {
        match self {
            EdgeStyle::None => "none",
            EdgeStyle::Line => "line",
            EdgeStyle::Polyline => "polyline",
            EdgeStyle::Curved => "curved",
            EdgeStyle::Ortho => "ortho",
            EdgeStyle::Spline => "spline",
        }
    }
}

#[derive(Clone)]
pub struct Style {
    pub schedule_rankdir: RankDir,
    pub edge_style: EdgeStyle,

    pub fontname: String,

    pub color_background: String,
    pub color_schedule: String,
    pub color_schedule_label: String,
    pub color_schedule_border: String,
    pub color_edge: Vec<String>,

    pub penwidth_edge: f32,
}
// colors are from https://iamkate.com/data/12-bit-rainbow/, without the #cc6666
impl Style {
    pub fn light() -> Style {
        Style {
            schedule_rankdir: RankDir::default(),
            edge_style: EdgeStyle::default(),
            fontname: "Helvetica".into(),
            color_background: "white".into(),
            color_schedule: "#00000008".into(),
            color_schedule_border: "#00000040".into(),
            color_schedule_label: "#000000".into(),
            color_edge: vec![
                "#eede00".into(),
                "#881877".into(),
                "#00b0cc".into(),
                "#aa3a55".into(),
                "#44d488".into(),
                "#0090cc".into(),
                "#ee9e44".into(),
                "#663699".into(),
                "#3363bb".into(),
                "#22c2bb".into(),
                "#99d955".into(),
            ],
            penwidth_edge: 2.0,
        }
    }

    pub fn dark_discord() -> Style {
        Style {
            schedule_rankdir: RankDir::default(),
            edge_style: EdgeStyle::default(),
            fontname: "Helvetica".into(),
            color_background: "#35393f".into(),
            color_schedule: "#ffffff44".into(),
            color_schedule_border: "#ffffff50".into(),
            color_schedule_label: "#ffffff".into(),
            color_edge: vec![
                "#eede00".into(),
                "#881877".into(),
                "#00b0cc".into(),
                "#aa3a55".into(),
                "#44d488".into(),
                "#0090cc".into(),
                "#ee9e44".into(),
                "#663699".into(),
                "#3363bb".into(),
                "#22c2bb".into(),
                "#99d955".into(),
            ],
            penwidth_edge: 2.0,
        }
    }

    pub fn dark_github() -> Style {
        Style {
            schedule_rankdir: RankDir::default(),
            edge_style: EdgeStyle::default(),
            fontname: "Helvetica".into(),
            color_background: "#0d1117".into(),
            color_schedule: "#ffffff44".into(),
            color_schedule_border: "#ffffff50".into(),
            color_schedule_label: "#ffffff".into(),
            color_edge: vec![
                "#eede00".into(),
                "#881877".into(),
                "#00b0cc".into(),
                "#aa3a55".into(),
                "#44d488".into(),
                "#0090cc".into(),
                "#ee9e44".into(),
                "#663699".into(),
                "#3363bb".into(),
                "#22c2bb".into(),
                "#99d955".into(),
            ],
            penwidth_edge: 2.0,
        }
    }
}
impl Default for Style {
    fn default() -> Self {
        Style::dark_github()
    }
}

pub struct NodeStyle {
    pub bg_color: String,
    pub text_color: String,
    pub border_color: String,
    pub border_width: String,
}

// Function that maps `System` to `T`
type SystemMapperFn<T> = Box<dyn Fn(&dyn System<In = (), Out = ()>) -> T>;

// Function that maps `ComponentInfo` to `T`
type ComponentInfoMapperFn<T> = Box<dyn Fn(&ComponentInfo) -> T>;

// Function that maps `Schedule` to `T`
type ScheduleMapperFn<T> = Box<dyn Fn(&Schedule) -> T>;

pub struct Settings {
    pub style: Style,
    pub system_style: SystemMapperFn<SystemStyle>,
    pub event_style: ComponentInfoMapperFn<ComponentInfoStyle>,

    /// When set to `Some`, will only include systems matching the predicate, and their ancestor sets
    pub include_system: Option<SystemMapperFn<bool>>,
    pub include_schedule: Option<ScheduleMapperFn<bool>>,

    pub prettify_system_names: bool,
}

impl Settings {
    /// Set the `include_system` predicate to match only systems for which their names matches `filter`
    pub fn filter_name(mut self, filter: impl Fn(&str) -> bool + 'static) -> Self {
        self.include_system = Some(Box::new(move |system| {
            let name = system.name();
            filter(&name)
        }));
        self
    }
    /// Set the `include_system` predicate to only match systems from the specified crate
    pub fn filter_in_crate(mut self, crate_: &str) -> Self {
        let crate_ = crate_.to_owned();
        self.include_system = Some(Box::new(move |system| {
            let name = system.name();
            name.starts_with(&crate_)
        }));
        self
    }
    /// Set the `include_system` predicate to only match systems from the specified crates
    pub fn filter_in_crates(mut self, crates: &[&str]) -> Self {
        let crates: Vec<_> = crates.iter().map(|&s| s.to_owned()).collect();
        self.include_system = Some(Box::new(move |system| {
            let name = system.name();
            crates.iter().any(|crate_| name.starts_with(crate_))
        }));
        self
    }

    pub fn get_system_style(&self, system: &dyn System<In = (), Out = ()>) -> NodeStyle {
        let style = (self.system_style)(system);

        // Check if bg is dark
        let [h, s, l, _] = style.bg_color.as_hsla_f32();
        // TODO Fix following: https://ux.stackexchange.com/q/107318
        let is_dark = l < 0.6;

        // Calculate text color based on bg
        let text_color = style.text_color.unwrap_or_else(|| {
            if is_dark {
                Color::hsl(h, s, 0.9)
            } else {
                Color::hsl(h, s, 0.1)
            }
        });

        // Calculate border color based on bg
        let border_color = style.border_color.unwrap_or_else(|| {
            let offset = if is_dark { 0.2 } else { -0.2 };
            let border_l = (l + offset).clamp(0.0, 1.0);

            Color::hsl(h, s, border_l)
        });

        NodeStyle {
            bg_color: color_to_hex(style.bg_color),
            text_color: color_to_hex(text_color),
            border_color: color_to_hex(border_color),
            border_width: style.border_width.to_string(),
        }
    }

    pub fn get_event_style(&self, system: &ComponentInfo) -> NodeStyle {
        let style = (self.event_style)(system);

        // Check if bg is dark
        let [h, s, l, _] = style.bg_color.as_hsla_f32();
        // TODO Fix following: https://ux.stackexchange.com/q/107318
        let is_dark = l < 0.6;

        // Calculate text color based on bg
        let text_color = style.text_color.unwrap_or_else(|| {
            if is_dark {
                Color::hsl(h, s, 0.9)
            } else {
                Color::hsl(h, s, 0.1)
            }
        });

        // Calculate border color based on bg
        let border_color = style.border_color.unwrap_or_else(|| {
            let offset = if is_dark { 0.2 } else { -0.2 };
            let border_l = (l + offset).clamp(0.0, 1.0);

            Color::hsl(h, s, border_l)
        });

        NodeStyle {
            bg_color: color_to_hex(style.bg_color),
            text_color: color_to_hex(text_color),
            border_color: color_to_hex(border_color),
            border_width: style.border_width.to_string(),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            style: Style::default(),
            system_style: Box::new(system_to_style),
            event_style: Box::new(event_to_style),

            include_system: Some(Box::new(exclude_bevy_event_update_system)),
            include_schedule: Some(Box::new(base_schedule_update)),

            prettify_system_names: true,
        }
    }
}

pub fn exclude_bevy_event_update_system(system: &dyn System<In = (), Out = ()>) -> bool {
    !system
        .name()
        .starts_with("bevy_ecs::event::event_update_system<")
}
pub fn base_schedule_update(schedule: &Schedule) -> bool {
    let labels: Vec<Box<dyn ScheduleLabel>> = vec![
        Box::new(First),
        Box::new(PreUpdate),
        Box::new(Update),
        Box::new(PostUpdate),
    ];
    labels
        .iter()
        .any(|s| (*schedule.label().0).as_dyn_eq().dyn_eq((**s).as_dyn_eq()))
}
