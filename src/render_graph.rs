use std::collections::HashMap;

use crate::dot::{font_tag, html_escape, DotGraph};
use bevy_render::render_graph::{Edge, NodeId, RenderGraph};
use pretty_type_name::pretty_type_name_str;

use self::iter_utils::EitherOrBoth;

/// Formats the render graph into a dot graph.
pub fn render_graph_dot(graph: &RenderGraph) -> String {
    render_graph_dot_styled(graph, &RenderGraphStyle::default())
}
pub struct RenderGraphStyle {
    pub fontname: String,
    pub fontsize: f32,
    pub textcolor: String,
    pub typename_color: String,
    pub background_color: String,
    pub subgraph_background_color: Vec<String>,
    pub subgraph_label_font_color: String,
    pub node_color: String,
    pub node_style: String,
    pub edge_color: String,
    pub slot_edge_color: String,
}
impl RenderGraphStyle {
    pub fn light() -> Self {
        RenderGraphStyle {
            fontname: "Helvetica".into(),
            fontsize: 14.0,
            textcolor: "black".into(),
            typename_color: "red".into(),
            background_color: "white".into(),
            subgraph_background_color: vec!["#e4e9f5".into(), "#c4d0ed".into()],
            subgraph_label_font_color: "black".into(),
            node_color: "black".into(),
            node_style: "rounded".into(),
            edge_color: "black".into(),
            slot_edge_color: "blue".into(),
        }
    }

    pub fn dark() -> Self {
        RenderGraphStyle {
            fontname: "Helvetica".into(),
            fontsize: 14.0,
            textcolor: "white".into(),
            typename_color: "red".into(),
            background_color: "#35393F".into(),
            subgraph_background_color: vec!["#5e6570".into(), "#6a83aa".into()],
            subgraph_label_font_color: "black".into(),
            node_color: "#99aab5".into(),
            node_style: "rounded".into(),
            edge_color: "white".into(),
            slot_edge_color: "white".into(),
        }
    }
}

impl Default for RenderGraphStyle {
    fn default() -> Self {
        RenderGraphStyle::dark()
    }
}

/// Formats the render graph into a dot graph using a custom [RenderGraphStyle].
pub fn render_graph_dot_styled(graph: &RenderGraph, style: &RenderGraphStyle) -> String {
    let options = [("rankdir", "LR"), ("ranksep", "1.0")];
    let mut dot = DotGraph::digraph("RenderGraph", &options)
        .graph_attributes(&[("bgcolor", &style.background_color)])
        .edge_attributes(&[
            ("fontname", &style.fontname),
            ("fontcolor", &style.textcolor),
        ])
        .node_attributes(&[
            ("shape", "plaintext"),
            ("fontname", &style.fontname),
            ("fontcolor", &style.textcolor),
        ]);

    build_dot_graph(&mut dot, None, graph, style, 0);
    dot.finish()
}

fn sorted<'a, T: 'a, U: Ord>(
    iter: impl IntoIterator<Item = T>,
    key: impl Fn(&T) -> U,
) -> impl IntoIterator<Item = T> + 'a {
    let mut vec: Vec<_> = iter.into_iter().collect();
    vec.sort_by_key(key);
    vec.into_iter()
}

fn build_dot_graph(
    dot: &mut DotGraph,
    graph_name: Option<&str>,
    graph: &RenderGraph,
    style: &RenderGraphStyle,
    subgraph_nest_level: usize,
) {
    let node_mapping: HashMap<_, _> = graph
        .iter_nodes()
        .map(|node| {
            let name = format!(
                "{}{}",
                graph_name.unwrap_or(""),
                node.name.as_deref().unwrap_or(node.type_name)
            );
            (node.id, name)
        })
        .collect();

    // Convert to format fitting GraphViz node id requirements
    let node_id = |id: &NodeId| format!("{}_{}", graph_name.unwrap_or_default(), &node_mapping[id]);

    let mut nodes: Vec<_> = graph.iter_nodes().collect();
    nodes.sort_by_key(|node_state| &node_state.type_name);

    for (name, subgraph) in sorted(graph.iter_sub_graphs(), |(name, _)| *name) {
        let internal_name = format!("{}_{}", graph_name.unwrap_or_default(), name);
        let options = [("label", name)];
        let bg_color = &style.subgraph_background_color
            [subgraph_nest_level % style.subgraph_background_color.len()];
        let mut sub_dot = DotGraph::subgraph(&internal_name, &options).graph_attributes(&[
            ("style", "rounded,filled"),
            ("color", bg_color),
            ("fontcolor", &style.subgraph_label_font_color),
        ]);
        build_dot_graph(
            &mut sub_dot,
            Some(&internal_name),
            subgraph,
            style,
            subgraph_nest_level + 1,
        );
        dot.add_sub_graph(sub_dot);
    }

    for node in &nodes {
        let name = node.name.as_deref().unwrap_or("<node>");
        let type_name = pretty_type_name_str(node.type_name);

        let inputs = node
            .input_slots
            .iter()
            .enumerate()
            .map(|(index, slot)| {
                format!(
                    "<TD PORT=\"{}\">{}: {}</TD>",
                    html_escape(&format!("in-{}", index)),
                    html_escape(&slot.name),
                    html_escape(&format!("{:?}", slot.slot_type))
                )
            })
            .collect::<Vec<_>>();

        let outputs = node
            .output_slots
            .iter()
            .enumerate()
            .map(|(index, slot)| {
                format!(
                    "<TD PORT=\"{}\">{}: {}</TD>",
                    html_escape(&format!("out-{}", index)),
                    html_escape(&slot.name),
                    html_escape(&format!("{:?}", slot.slot_type))
                )
            })
            .collect::<Vec<_>>();

        let slots = iter_utils::zip_longest(inputs.iter(), outputs.iter())
            .map(|pair| match pair {
                EitherOrBoth::Both(input, output) => format!("<TR>{}{}</TR>", input, output),
                EitherOrBoth::Left(input) => {
                    format!("<TR>{}<TD BORDER=\"0\">&nbsp;</TD></TR>", input)
                }
                EitherOrBoth::Right(output) => {
                    format!("<TR><TD BORDER=\"0\">&nbsp;</TD>{}</TR>", output)
                }
            })
            .collect::<String>();

        let label = format!(
            "<<TABLE STYLE=\"rounded\"><TR><TD PORT=\"title\" BORDER=\"0\" COLSPAN=\"2\">{}<BR/>{}</TD></TR>{}</TABLE>>",
            html_escape(name),
            font_tag(&type_name, &style.typename_color, 10),
            slots,
        );

        dot.add_node(
            &node_id(&node.id),
            &[
                ("label", &label),
                ("color", &style.node_color),
                ("fillcolor", &style.node_color),
            ],
        );
    }

    for node in &nodes {
        for edge in node.edges.input_edges() {
            match edge {
                Edge::SlotEdge {
                    input_node,
                    input_index,
                    output_node,
                    output_index,
                } => {
                    dot.add_edge_with_ports(
                        &node_id(output_node),
                        Some(&format!("out-{}:e", output_index)),
                        &node_id(input_node),
                        Some(&format!("in-{}:w", input_index)),
                        &[("color", &style.slot_edge_color)],
                    );
                }
                Edge::NodeEdge {
                    input_node,
                    output_node,
                } => {
                    dot.add_edge_with_ports(
                        &node_id(output_node),
                        Some("title:e"),
                        &node_id(input_node),
                        Some("title:w"),
                        &[("color", &style.edge_color)],
                    );
                }
            }
        }
    }
}

mod iter_utils {
    use std::iter::Fuse;

    pub enum EitherOrBoth<A, B> {
        Both(A, B),
        Left(A),
        Right(B),
    }

    #[derive(Clone, Debug)]
    pub struct ZipLongest<T, U> {
        a: Fuse<T>,
        b: Fuse<U>,
    }

    pub fn zip_longest<T, U>(a: T, b: U) -> ZipLongest<T, U>
    where
        T: Iterator,
        U: Iterator,
    {
        ZipLongest {
            a: a.fuse(),
            b: b.fuse(),
        }
    }

    impl<T, U> Iterator for ZipLongest<T, U>
    where
        T: Iterator,
        U: Iterator,
    {
        type Item = EitherOrBoth<T::Item, U::Item>;

        fn next(&mut self) -> Option<Self::Item> {
            match (self.a.next(), self.b.next()) {
                (None, None) => None,
                (Some(a), None) => Some(EitherOrBoth::Left(a)),
                (None, Some(b)) => Some(EitherOrBoth::Right(b)),
                (Some(a), Some(b)) => Some(EitherOrBoth::Both(a, b)),
            }
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            max_size_hint(self.a.size_hint(), self.b.size_hint())
        }
    }

    fn max_size_hint(
        a: (usize, Option<usize>),
        b: (usize, Option<usize>),
    ) -> (usize, Option<usize>) {
        let (a_lower, a_upper) = a;
        let (b_lower, b_upper) = b;

        let lower = std::cmp::max(a_lower, b_lower);

        let upper = match (a_upper, b_upper) {
            (Some(x), Some(y)) => Some(std::cmp::max(x, y)),
            _ => None,
        };

        (lower, upper)
    }
}
