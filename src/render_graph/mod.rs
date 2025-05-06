pub mod settings;

use bevy_ecs::intern::Interned;
pub use settings::Settings;

use self::iter_utils::sorted;
use crate::dot::{font_tag, html_escape, DotGraph};
use bevy_render::render_graph::{Edge, RenderGraph, RenderLabel};
use iter_utils::EitherOrBoth;
use std::collections::HashMap;

/// Formats the render graph into a dot graph
pub fn render_graph_dot(graph: &RenderGraph, settings: &Settings) -> String {
    let mut dot = DotGraph::digraph("RenderGraph", &[("rankdir", "LR"), ("ranksep", "1.0")])
        .graph_attributes(&[("bgcolor", &settings.style.color_background)])
        .edge_attributes(&[
            ("fontname", &settings.style.fontname),
            ("fontcolor", &settings.style.color_text),
        ])
        .node_attributes(&[
            ("shape", "plaintext"),
            ("fontname", &settings.style.fontname),
            ("fontcolor", &settings.style.color_text),
        ]);

    build_dot_graph(&mut dot, None, graph, settings, 0);
    dot.finish()
}

fn build_dot_graph(
    dot: &mut DotGraph,
    graph_name: Option<&str>,
    graph: &RenderGraph,
    settings: &Settings,
    subgraph_nest_level: usize,
) {
    let fmt_label = |label: Interned<dyn RenderLabel>| format!("{label:?}");

    let node_mapping: HashMap<_, _> = graph
        .iter_nodes()
        .map(|node| {
            (
                node.label,
                format!("{}{:?}", graph_name.unwrap_or_default(), node.label),
            )
        })
        .collect();

    // Convert to format fitting GraphViz node id requirements
    let node_id = |id: &Interned<dyn RenderLabel>| {
        format!("{}_{}", graph_name.unwrap_or_default(), &node_mapping[id])
    };

    let mut nodes: Vec<_> = graph.iter_nodes().collect();
    nodes.sort_by(|a, b| {
        a.type_name
            .cmp(b.type_name)
            .then_with(|| fmt_label(a.label).cmp(&fmt_label(b.label)))
    });

    let layer_style = &settings.style.layers[subgraph_nest_level % settings.style.layers.len()];
    let next_layer_style =
        &settings.style.layers[(subgraph_nest_level + 1) % settings.style.layers.len()];

    for (name, subgraph) in sorted(graph.iter_sub_graphs(), |(name, _)| format!("{name:?}")) {
        let internal_name = format!("{}_{:?}", graph_name.unwrap_or_default(), name);
        let mut sub_dot = DotGraph::subgraph(
            &internal_name,
            &[("label", &format!("{name:?}")), ("fontcolor", "red")],
        )
        .graph_attributes(&[
            ("style", "rounded,filled"),
            ("color", &next_layer_style.color_background),
            ("fontcolor", &next_layer_style.color_label),
        ]);
        build_dot_graph(
            &mut sub_dot,
            Some(&internal_name),
            subgraph,
            settings,
            subgraph_nest_level + 1,
        );
        dot.add_sub_graph(sub_dot);
    }

    for node in &nodes {
        let name = &fmt_label(node.label);
        let type_name = disqualified::ShortName(node.type_name);

        let inputs = node
            .input_slots
            .iter()
            .enumerate()
            .map(|(index, slot)| {
                format!(
                    "<TD PORT=\"{}\">{}: {}</TD>",
                    html_escape(&format!("in-{index}")),
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
                    html_escape(&format!("out-{index}")),
                    html_escape(&slot.name),
                    html_escape(&format!("{:?}", slot.slot_type))
                )
            })
            .collect::<Vec<_>>();

        let slots = iter_utils::zip_longest(inputs.iter(), outputs.iter())
            .map(|pair| match pair {
                EitherOrBoth::Both(input, output) => format!("<TR>{input}{output}</TR>"),
                EitherOrBoth::Left(input) => {
                    format!("<TR>{input}<TD BORDER=\"0\">&nbsp;</TD></TR>")
                }
                EitherOrBoth::Right(output) => {
                    format!("<TR><TD BORDER=\"0\">&nbsp;</TD>{output}</TR>")
                }
            })
            .collect::<String>();

        let label = format!(
            "RAW:<<TABLE STYLE=\"rounded\"><TR><TD PORT=\"title\" BORDER=\"0\" COLSPAN=\"2\">{}<BR/>{}</TD></TR>{}</TABLE>>",
            html_escape(name),
            font_tag(&type_name.to_string(), &settings.style.color_typename, 10),
            slots,
        );

        dot.add_node(
            &node_id(&node.label),
            &[
                ("label", &label),
                ("color", &settings.style.color_node),
                ("fillcolor", &settings.style.color_node),
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
                        Some(&format!("out-{output_index}:e")),
                        &node_id(input_node),
                        Some(&format!("in-{input_index}:w")),
                        &[("color", &layer_style.color_edge_slot)],
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
                        &[("color", &layer_style.color_edge)],
                    );
                }
            }
        }
    }
}
mod iter_utils {
    use std::iter::Fuse;

    pub fn sorted<'a, T: 'a, U: Ord>(
        iter: impl IntoIterator<Item = T>,
        key: impl Fn(&T) -> U,
    ) -> impl IntoIterator<Item = T> + 'a {
        let mut vec: Vec<_> = iter.into_iter().collect();
        vec.sort_by_key(key);
        vec.into_iter()
    }

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
