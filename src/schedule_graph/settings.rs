use bevy_ecs::system::System;

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
    pub color_system: String,
    pub color_system_border: String,
    pub color_set: String,
    pub color_set_border: String,
    pub color_edge: String,
    pub multiple_set_edge_color: String,

    pub ambiguity_color: String,
    pub ambiguity_bgcolor: String,
}
impl Style {
    pub fn light() -> Style {
        Style {
            schedule_rankdir: RankDir::default(),
            edge_style: EdgeStyle::default(),
            fontname: "Helvetica".into(),
            color_background: "white".into(),
            color_system: "white".into(),
            color_system_border: "black".into(),
            color_set: "white".into(),
            color_set_border: "black".into(),
            color_edge: "black".into(),
            multiple_set_edge_color: "blue".into(),
            ambiguity_color: "#c93526".into(),
            ambiguity_bgcolor: "#d3d3d3".into(),
        }
    }

    pub fn dark_discord() -> Style {
        Style {
            schedule_rankdir: RankDir::default(),
            edge_style: EdgeStyle::default(),
            fontname: "Helvetica".into(),
            color_background: "#35393f".into(),
            color_system: "#eff1f3".into(),
            color_system_border: "#eff1f3".into(),
            color_set: "#99aab5".into(),
            color_set_border: "black".into(),
            color_edge: "white".into(),
            ambiguity_color: "#c93526".into(),
            ambiguity_bgcolor: "#c5daeb".into(),
            multiple_set_edge_color: "blue".into(),
        }
    }

    pub fn dark_github() -> Style {
        Style {
            schedule_rankdir: RankDir::default(),
            edge_style: EdgeStyle::default(),
            fontname: "Helvetica".into(),
            color_background: "#0d1117".into(),
            color_system: "#eff1f3".into(),
            color_system_border: "#eff1f3".into(),
            color_set: "#6f90ad".into(),
            color_set_border: "black".into(),
            color_edge: "white".into(),
            ambiguity_color: "#c93526".into(),
            ambiguity_bgcolor: "#c6e6ff".into(),
            multiple_set_edge_color: "blue".into(),
        }
    }
}
impl Default for Style {
    fn default() -> Self {
        Style::dark_github()
    }
}

pub struct Settings {
    pub style: Style,

    /// When set to `Some`, will only include systems matching the predicate, and their ancestor sets
    pub include_system: Option<Box<dyn Fn(&dyn System<In = (), Out = ()>) -> bool>>,
    pub collapse_single_system_sets: bool,

    pub ambiguity_enable: bool,
    pub ambiguity_enable_on_world: bool,

    pub prettify_system_names: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            style: Style::default(),

            include_system: None,
            collapse_single_system_sets: false,

            ambiguity_enable: true,
            ambiguity_enable_on_world: false,

            prettify_system_names: true,
        }
    }
}
