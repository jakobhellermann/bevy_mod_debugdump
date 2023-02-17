#[derive(Default)]
pub struct Settings {
    pub style: Style,
}

pub struct LayerStyle {
    pub color_background: String,
    pub color_label: String,
    pub color_edge: String,
    pub color_edge_slot: String,
}

pub struct Style {
    pub fontname: String,
    pub fontsize: f32,
    pub color_background: String,
    pub color_text: String,
    pub color_typename: String,
    pub layers: Vec<LayerStyle>,
    pub color_node: String,
    pub node_style: String,
}
impl Style {
    pub fn light() -> Self {
        Style {
            fontname: "Helvetica".into(),
            fontsize: 14.0,
            color_text: "black".into(),
            color_typename: "red".into(),
            color_background: "white".into(),
            layers: vec![
                LayerStyle {
                    color_background: "#c4d0ed".into(),
                    color_edge: "black".into(),
                    color_edge_slot: "blue".into(),
                    color_label: "black".into(),
                },
                LayerStyle {
                    color_background: "#e4e9f5".into(),
                    color_label: "black".into(),
                    color_edge: "black".into(),
                    color_edge_slot: "blue".into(),
                },
            ],
            color_node: "black".into(),
            node_style: "rounded".into(),
        }
    }

    pub fn dark_github() -> Self {
        Style {
            fontname: "Helvetica".into(),
            fontsize: 14.0,
            color_text: "white".into(),
            color_typename: "red".into(),
            color_background: "#0d1117".into(),
            layers: vec![
                LayerStyle {
                    color_background: "#6f90ad".into(),
                    color_label: "black".into(),
                    color_edge: "white".into(),
                    color_edge_slot: "#715ed6".into(),
                },
                LayerStyle {
                    color_background: "#343a42".into(),
                    color_label: "white".into(),
                    color_edge: "white".into(),
                    color_edge_slot: "#a79be6".into(),
                },
            ],
            color_node: "white".into(),
            node_style: "rounded".into(),
        }
    }

    pub fn dark_discord() -> Self {
        Style {
            fontname: "Helvetica".into(),
            fontsize: 14.0,
            color_text: "white".into(),
            color_typename: "red".into(),
            color_background: "#35393F".into(),
            layers: vec![
                LayerStyle {
                    color_background: "#6f90ad".into(),
                    color_label: "black".into(),
                    color_edge: "white".into(),
                    color_edge_slot: "#a79be6".into(),
                },
                LayerStyle {
                    color_background: "#5e6570".into(),
                    color_label: "white".into(),
                    color_edge: "white".into(),
                    color_edge_slot: "#a79be6".into(),
                },
            ],
            color_node: "#99aab5".into(),
            node_style: "rounded".into(),
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Style::dark_github()
    }
}
