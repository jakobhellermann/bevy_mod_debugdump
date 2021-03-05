use std::borrow::Cow;

use crate::{
    dot::{font_tag, DotGraph},
    utils,
};
use bevy::render::render_graph::{Edge, NodeId, RenderGraph};
use itertools::{EitherOrBoth, Itertools};

/// Escape tags in such a way that it is suitable for inclusion in a
/// Graphviz HTML label.
pub fn escape_html<'a, S>(s: S) -> Cow<'a, str>
where
    S: Into<Cow<'a, str>>,
{
    s.into()
        .replace("&", "&amp;")
        .replace("\"", "&quot;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .into()
}

pub fn render_graph_dot(graph: &RenderGraph) -> String {
    let options = [("rankdir", "LR"), ("ranksep", "1.0")];
    let mut dot = DotGraph::new("RenderGraph", &options);

    // Convert to format fitting GraphViz node id requirements
    let node_id = |id: &NodeId| format!("{}", id.uuid().as_u128());
    let font = ("fontname", "Roboto");
    let shape = ("shape", "plaintext");
    let edge_color = ("color", "\"blue\"");

    dot.edge_attributes(&[font]).node_attributes(&[shape, font]);

    let mut nodes: Vec<_> = graph.iter_nodes().collect();
    nodes.sort_by_key(|node_state| &node_state.type_name);

    for node in &nodes {
        let name = node.name.as_deref().unwrap_or("<node>");
        let type_name = utils::short_name(node.type_name);

        let inputs = node
            .input_slots
            .iter()
            .enumerate()
            .map(|(index, slot)| {
                format!(
                    "<TD PORT=\"{}\">{}: {}</TD>",
                    escape_html(format!("{}", index)),
                    escape_html(slot.info.name.clone()),
                    escape_html(format!("{:?}", slot.info.resource_type))
                )
            })
            .collect::<Vec<_>>();

        let outputs = node
            .output_slots
            .iter()
            .enumerate()
            .map(|(index, slot)| {
                format!(
                    "<TD PORT=\"{}\">{}: {:?}</TD>",
                    escape_html(format!("{}", index)),
                    escape_html(slot.info.name.clone()),
                    escape_html(format!("{:?}", slot.info.resource_type))
                )
            })
            .collect::<Vec<_>>();

        let slots = inputs
            .iter()
            .zip_longest(outputs.iter())
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
            "<<TABLE><TR><TD PORT=\"title\" BORDER=\"0\" COLSPAN=\"2\">{}<BR/>{}</TD></TR>{}</TABLE>>",
            escape_html(name),
            font_tag(&escape_html(&type_name), "red", 10),
            slots,
        );

        dot.add_node(&node_id(&node.id), &[("label", &label)]);
    }

    for node in graph.iter_nodes() {
        for edge in &node.edges.input_edges {
            match edge {
                Edge::SlotEdge {
                    input_node,
                    input_index,
                    output_node,
                    output_index,
                } => {
                    let input = graph.get_node_state(*input_node).unwrap();
                    let input_slot = &input.input_slots.iter().nth(*input_index).unwrap().info;
                    let label = format!("\"{}\"", input_slot.name);

                    dot.add_edge(
                        &node_id(output_node),
                        Some(&format!("{}:e", input_index)),
                        &node_id(input_node),
                        Some(&format!("{}:w", output_index)),
                        &[("label", &label), edge_color],
                    );
                }
                Edge::NodeEdge {
                    input_node,
                    output_node,
                } => {
                    dot.add_edge(
                        &node_id(output_node),
                        Some("title:e"),
                        &node_id(input_node),
                        Some("title:w"),
                        &[("style", "dashed")],
                    );
                }
            }
        }
    }

    dot.finish()
}
