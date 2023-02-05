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
            color_set_border: "white".into(),
            color_edge: "black".into(),
            multiple_set_edge_color: "red".into(),
            ambiguity_color: "blue".into(),
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
            color_set_border: "white".into(),
            color_edge: "white".into(),
            ambiguity_color: "blue".into(),
            ambiguity_bgcolor: "#d3d3d3".into(),
            multiple_set_edge_color: "red".into(),
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
            color_set_border: "white".into(),
            color_edge: "white".into(),
            ambiguity_color: "#c93526".into(),
            ambiguity_bgcolor: "#C6E6FF".into(),
            multiple_set_edge_color: "red".into(),
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

    pub include_system: Box<dyn Fn(&dyn System<In = (), Out = ()>) -> bool>,
    pub include_single_system_in_set: bool,

    pub ambiguity_enable: bool,
    pub ambiguity_enable_on_world: bool,

    pub prettify_system_names: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            style: Style::default(),

            include_system: Box::new(|_| true),
            include_single_system_in_set: true,

            ambiguity_enable: false,
            ambiguity_enable_on_world: false,

            prettify_system_names: true,
        }
    }
}

impl Settings {
    pub(crate) fn include_system(&self, system: &dyn System<In = (), Out = ()>) -> bool {
        (self.include_system)(system)
    }
}
