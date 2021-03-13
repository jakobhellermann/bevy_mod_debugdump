use crate::dot::{font_tag, html_escape, DotGraph};
use bevy::{
    reflect::TypeRegistration,
    render::render_graph::{Edge, NodeId, RenderGraph},
};
use itertools::{EitherOrBoth, Itertools};

pub fn render_graph_dot(graph: &RenderGraph) -> String {
    let options = [("rankdir=LR"), ("ranksep=1.0")];
    let mut dot = DotGraph::digraph("RenderGraph", &options);

    // Convert to format fitting GraphViz node id requirements
    let node_id = |id: &NodeId| format!("{}", id.uuid().as_u128());
    let font = ("fontname", "Helvetica");
    let shape = ("shape", "plaintext");
    let edge_color = ("color", "\"blue\"");

    dot.edge_attributes(&[font]).node_attributes(&[shape, font]);

    let mut nodes: Vec<_> = graph.iter_nodes().collect();
    nodes.sort_by_key(|node_state| &node_state.type_name);

    for node in &nodes {
        let name = node.name.as_deref().unwrap_or("<node>");
        let type_name = TypeRegistration::get_short_name(node.type_name);

        let inputs = node
            .input_slots
            .iter()
            .enumerate()
            .map(|(index, slot)| {
                format!(
                    "<TD PORT=\"{}\">{}: {}</TD>",
                    html_escape(&format!("{}", index)),
                    html_escape(&slot.info.name),
                    html_escape(&format!("{:?}", slot.info.resource_type))
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
                    html_escape(&format!("{}", index)),
                    html_escape(&slot.info.name),
                    html_escape(&format!("{:?}", slot.info.resource_type))
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
            "<<TABLE STYLE=\"rounded\"><TR><TD PORT=\"title\" BORDER=\"0\" COLSPAN=\"2\">{}<BR/>{}</TD></TR>{}</TABLE>>",
            html_escape(name),
            font_tag(&type_name, "red", 10),
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
                    dot.add_edge_with_ports(
                        &node_id(output_node),
                        Some(&format!("{}:e", output_index)),
                        &node_id(input_node),
                        Some(&format!("{}:w", input_index)),
                        &[edge_color],
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
                        &[],
                    );
                }
            }
        }
    }

    dot.finish()
}
