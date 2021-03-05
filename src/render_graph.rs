use crate::{
    dot::{font_tag, DotGraph},
    utils,
};
use bevy::render::render_graph::{Edge, NodeId, RenderGraph};

pub fn render_graph_dot(graph: &RenderGraph) -> String {
    let options = [("rankdir", "LR"), ("ranksep", "1.0")];
    let mut dot = DotGraph::new("RenderGraph", &options);

    let node_id = |id: &NodeId| format!("\"{:?}\"", id);
    let font = ("fontname", "Roboto");
    let edge_color = ("color", "\"blue\"");

    dot.edge_attributes(&[font]).node_attributes(&[font]);

    let mut nodes: Vec<_> = graph.iter_nodes().collect();
    nodes.sort_by_key(|node_state| &node_state.type_name);

    for node in &nodes {
        let name = node.name.as_deref().unwrap_or("<node>");
        let type_name = utils::short_name(node.type_name);

        let outputs = node
            .output_slots
            .iter()
            .map(|slot| format!("{}:{:?}, ", slot.info.name, slot.info.resource_type))
            .collect::<String>();
        let outputs = outputs.trim_end_matches(", ");
        let outputs = match outputs.is_empty() {
            false => format!("<BR/> {}", font_tag(&format!("-> {}", outputs), "blue", 10)),
            true => "".into(),
        };

        let label = format!(
            "<{}<BR />{} {}>",
            name,
            font_tag(&type_name, "red", 10),
            outputs,
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
                    output_index: _,
                } => {
                    let input = graph.get_node_state(*input_node).unwrap();
                    let input_slot = &input.input_slots.iter().nth(*input_index).unwrap().info;
                    let label = format!("\"{}\"", input_slot.name);

                    dot.add_edge(
                        &node_id(output_node),
                        &node_id(input_node),
                        &[("label", &label), edge_color],
                    );
                }
                Edge::NodeEdge {
                    input_node,
                    output_node,
                } => {
                    dot.add_edge(&node_id(output_node), &node_id(input_node), &[]);
                }
            }
        }
    }

    dot.finish()
}
