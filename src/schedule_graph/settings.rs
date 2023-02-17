use std::any::TypeId;

use bevy_ecs::{component::ComponentId, system::System, world::World};

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
    pub color_edge: Vec<String>,
    pub multiple_set_edge_color: String,

    pub ambiguity_color: String,
    pub ambiguity_bgcolor: String,

    pub penwidth_edge: f32,
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
            color_edge: vec![
                "#c4cc1d".into(),
                "#251dcc".into(),
                "#cd1d91".into(),
                "#1dcd59".into(),
                "#1dc4cc".into(),
                "#cc251d".into(),
                "#ccd60d".into(),
                "#29e000".into(),
                "#58cd0c".into(),
                "#810ccd".into(),
                "#cf410c".into(),
                "#0c9acf".into(),
            ],
            multiple_set_edge_color: "blue".into(),
            ambiguity_color: "#c93526".into(),
            ambiguity_bgcolor: "#d3d3d3".into(),
            penwidth_edge: 2.0,
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
            color_edge: vec![
                "#a6f2d4".into(),
                "#c4cc1d".into(),
                "#251dcc".into(),
                "#cd1d91".into(),
                "#1dcd59".into(),
                "#1dc4cc".into(),
                "#cc251d".into(),
                "#ccd60d".into(),
                "#29e000".into(),
                "#58cd0c".into(),
                "#810ccd".into(),
                "#cf410c".into(),
                "#0c9acf".into(),
            ],
            ambiguity_color: "#c93526".into(),
            ambiguity_bgcolor: "#c5daeb".into(),
            multiple_set_edge_color: "blue".into(),
            penwidth_edge: 2.0,
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
            color_edge: vec![
                "#a6f2d4".into(),
                "#c4cc1d".into(),
                "#251dcc".into(),
                "#cd1d91".into(),
                "#1dcd59".into(),
                "#1dc4cc".into(),
                "#cc251d".into(),
                "#ccd60d".into(),
                "#29e000".into(),
                "#58cd0c".into(),
                "#810ccd".into(),
                "#cf410c".into(),
                "#0c9acf".into(),
            ],
            ambiguity_color: "#c93526".into(),
            ambiguity_bgcolor: "#c6e6ff".into(),
            multiple_set_edge_color: "blue".into(),
            penwidth_edge: 2.0,
        }
    }
}
impl Default for Style {
    fn default() -> Self {
        Style::dark_github()
    }
}

type IncludeAmbiguityFn = dyn Fn(
    &dyn System<In = (), Out = ()>,
    &dyn System<In = (), Out = ()>,
    &[ComponentId],
    &World,
) -> bool;

pub struct Settings {
    pub style: Style,

    /// When set to `Some`, will only include systems matching the predicate, and their ancestor sets
    pub include_system: Option<Box<dyn Fn(&dyn System<In = (), Out = ()>) -> bool>>,
    pub collapse_single_system_sets: bool,

    pub ambiguity_enable: bool,
    pub ambiguity_enable_on_world: bool,
    pub include_ambiguity: Option<Box<IncludeAmbiguityFn>>,

    pub prettify_system_names: bool,
}

impl Settings {
    pub fn without_single_ambiguities_on(mut self, type_ids: &[TypeId]) -> Self {
        let type_ids = type_ids.to_vec();
        self.include_ambiguity = Some(Box::new(move |_, _, conflicts, world| {
            let &[conflict] = conflicts else { return true };
            let Some(type_id) = world.components().get_info(conflict).and_then(|info| info.type_id()) else { return true };
            !type_ids.contains(&type_id)
        }));
        self
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            style: Style::default(),

            include_system: None,
            collapse_single_system_sets: false,

            ambiguity_enable: true,
            ambiguity_enable_on_world: false,
            include_ambiguity: None,

            prettify_system_names: true,
        }
    }
}
