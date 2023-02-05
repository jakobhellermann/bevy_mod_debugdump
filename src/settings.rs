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

    pub multiple_set_edge_color: String,
    pub ambiguity_color: String,
    pub ambiguity_bgcolor: String,
}
impl Default for Style {
    fn default() -> Self {
        Style {
            schedule_rankdir: RankDir::default(),
            edge_style: EdgeStyle::default(),
            multiple_set_edge_color: "red".into(),
            ambiguity_color: "blue".into(),
            ambiguity_bgcolor: "#d3d3d3".into(),
        }
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
