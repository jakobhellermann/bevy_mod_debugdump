use std::any::TypeId;

use bevy_color::{Color, Hsla};
use bevy_ecs::{component::ComponentId, schedule::SystemSet, system::ScheduleSystem, world::World};

use super::system_style::{color_to_hex, system_to_style, SystemStyle};

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
    pub color_set: String,
    pub color_set_label: String,
    pub color_set_border: String,
    pub color_edge: Vec<String>,
    pub multiple_set_edge_color: String,

    pub ambiguity_color: String,
    pub ambiguity_bgcolor: String,

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
            color_set: "#00000008".into(),
            color_set_border: "#00000040".into(),
            color_set_label: "#000000".into(),
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
            color_set: "#ffffff44".into(),
            color_set_border: "#ffffff50".into(),
            color_set_label: "#ffffff".into(),
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
            color_set: "#ffffff44".into(),
            color_set_border: "#ffffff50".into(),
            color_set_label: "#ffffff".into(),
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

type IncludeAmbiguityFn = dyn Fn(&ScheduleSystem, &ScheduleSystem, &[ComponentId], &World) -> bool;

pub struct NodeStyle {
    pub bg_color: String,
    pub text_color: String,
    pub border_color: String,
    pub border_width: String,
}

// Function that maps `System` to `T`
type SystemMapperFn<T> = Box<dyn Fn(&ScheduleSystem) -> T>;

// Function that maps `SystemSet` to `T`
type SystemSetMapperFn<T> = Box<dyn Fn(&dyn SystemSet) -> T>;

pub struct Settings {
    pub style: Style,
    pub system_style: SystemMapperFn<SystemStyle>,

    /// When set to `Some`, will only include systems matching the predicate, and their ancestor sets
    pub include_system: Option<SystemMapperFn<bool>>,
    pub collapse_single_system_sets: bool,
    pub remove_transitive_edges: bool,

    pub ambiguity_enable: bool,
    pub ambiguity_enable_on_world: bool,
    pub include_ambiguity: Option<Box<IncludeAmbiguityFn>>,

    pub system_name: SystemMapperFn<String>,
    pub full_system_name: SystemMapperFn<String>,
    pub system_set_name: SystemSetMapperFn<String>,
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

    pub fn get_system_style(&self, system: &ScheduleSystem) -> NodeStyle {
        let style = (self.system_style)(system);

        // Check if bg is dark
        let Hsla {
            hue: h,
            saturation: s,
            lightness: l,
            alpha: _,
        } = Hsla::from(style.bg_color);
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

    /// Specifies `include_ambiguity` to ignore ambiguities that are only ambiguous with regard to `T`
    pub fn without_single_ambiguities_on<T: 'static>(mut self) -> Self {
        self.include_ambiguity = Some(Box::new(move |_, _, conflicts, world| {
            let &[conflict] = conflicts else { return true };
            let Some(type_id) = world
                .components()
                .get_info(conflict)
                .and_then(|info| info.type_id())
            else {
                return true;
            };
            type_id != TypeId::of::<T>()
        }));
        self
    }

    /// Specifies `include_ambiguity` to ignore ambiguities that are exactly one of the given `type_ids`
    pub fn without_single_ambiguities_on_one_of(mut self, type_ids: &[TypeId]) -> Self {
        let type_ids = type_ids.to_vec();
        self.include_ambiguity = Some(Box::new(move |_, _, conflicts, world| {
            let &[conflict] = conflicts else { return true };
            let Some(type_id) = world
                .components()
                .get_info(conflict)
                .and_then(|info| info.type_id())
            else {
                return true;
            };
            !type_ids.contains(&type_id)
        }));
        self
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            style: Style::default(),
            system_style: Box::new(system_to_style),

            include_system: None,
            collapse_single_system_sets: false,
            remove_transitive_edges: true,

            ambiguity_enable: false,
            ambiguity_enable_on_world: false,
            include_ambiguity: None,

            system_name: Box::new(pretty_system_name),
            full_system_name: Box::new(full_system_name),
            system_set_name: Box::new(default_system_set_name),
        }
    }
}

pub fn pretty_system_name(system: &ScheduleSystem) -> String {
    disqualified::ShortName(&system.name()).to_string()
}

pub fn full_system_name(system: &ScheduleSystem) -> String {
    system.name().to_string()
}

pub fn default_system_set_name(system_set: &dyn SystemSet) -> String {
    format!("{system_set:?}")
}
