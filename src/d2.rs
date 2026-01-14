#![cfg_attr(not(feature = "render_graph"), allow(dead_code))]

type NodeId = String;

use std::{borrow::Cow, fmt::Write as _, path::Display};

use bevy_ecs::entity_disabling::Disabled;
use bevy_platform::collections::HashMap;
use lexopt::Arg::Long;

#[derive(Debug)]
struct NodeRef(Vec<NodeId>);
impl NodeRef {
    pub fn prepend(mut self, value: NodeId) -> Self {
        self.0.insert(0, value);
        NodeRef(self.0)
    }
    pub fn to_string(&self) -> String {
        self.0
            .iter()
            .map(|x| escape_string(x))
            .collect::<Vec<_>>()
            .join(".")
    }
}

#[derive(Default)]
pub struct D2Graph {
    id: NodeId,

    nodes: Vec<(String, String)>,

    wip_edges: Vec<(String, String)>,
    wip_node_locations: HashMap<String, NodeRef>,

    sub_graphs: Vec<(NodeId, String, D2Graph)>,
}

fn escape_quote(input: &str) -> Cow<'_, str> {
    if input.contains('"') {
        Cow::Owned(input.replace('"', "\\\""))
    } else {
        Cow::Borrowed(input)
    }
}

fn escape_id(input: &str) -> Cow<'_, str> {
    if let Some(raw) = input.strip_prefix("RAW:") {
        return raw.into();
    }

    format!("\"{}\"", escape_quote(input)).into()
}
fn escape_string(input: &str) -> Cow<'_, str> {
    format!("\"{}\"", escape_quote(input)).into()
}

fn format_attributes(attrs: &[(&str, &str)]) -> String {
    let attrs: Vec<_> = attrs
        .iter()
        .map(|(a, b)| format!("{}={}", escape_id(a), escape_id(b)))
        .collect();
    let attrs = attrs.join(", ");
    format!("[{attrs}]")
}
pub fn font_tag(text: &str, color: &str, size: u8) -> String {
    if text.is_empty() {
        return "".to_string();
    }
    format!(
        "<FONT COLOR=\"{}\" POINT-SIZE=\"{}\">{}</FONT>",
        color,
        size,
        html_escape(text)
    )
}

pub fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('\"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

impl D2Graph {
    pub fn new(id: NodeId) -> D2Graph {
        D2Graph {
            id,
            ..Default::default()
        }
    }

    pub fn root() -> D2Graph {
        D2Graph::new("".into())
    }
    pub fn subgraph(&self, name: NodeId) -> D2Graph {
        D2Graph::new(name)
    }

    pub fn finish(&mut self) -> String {
        let mut buf = String::new();
        for (id, label) in &self.nodes {
            let _ = writeln!(&mut buf, "{}: {}", escape_string(&id), escape_string(label));
        }
        for (id, label, subgraph) in &mut self.sub_graphs {
            for (id, location) in std::mem::take(&mut subgraph.wip_node_locations) {
                assert!(self
                    .wip_node_locations
                    .insert(id, location.prepend(subgraph.id.clone()))
                    .is_none());
            }

            let _ = writeln!(
                &mut buf,
                "{}: {} {{\n  {}\n}}",
                escape_string(&id),
                escape_string(&label),
                subgraph.finish().trim().replace("\n", "\n  ")
            );
            self.wip_edges
                .extend(std::mem::take(&mut subgraph.wip_edges));
        }

        let is_root = self.id == "";
        if is_root {
            for edge in std::mem::take(&mut self.wip_edges) {
                dbg!(&self.wip_node_locations);
                dbg!(&edge.0);
                let from = &self.wip_node_locations.get(&edge.0);
                let to = &self.wip_node_locations.get(&edge.1);
                let (Some(from), Some(to)) = (from, to) else {
                    dbg!();
                    continue;
                };

                let _ = writeln!(&mut buf, "{} -> {}", from.to_string(), to.to_string(),);
            }
        }

        buf
    }

    pub fn add_sub_graph(&mut self, id: &str, label: &str, graph: D2Graph) {
        // let subgraph = graph.finish().replace('\n', "\n\t");
        // self.write_no_newline("\t");
        // self.write(subgraph);
        self.sub_graphs
            .push((id.to_owned(), label.to_owned(), graph));
        dbg!(&id);
        self.wip_node_locations
            .insert(id.to_owned(), NodeRef(vec![id.to_owned()]));
        /*self.write(format!(
            "{}: \"{}\" {{\n  {}\n}}\n",
            escape_id(id),
            escape_quote(label),
            graph.finish().trim().replace("\n", "\n  ")
        ));*/
    }

    // TODO name
    pub fn add_directive(&mut self, directive: &str, value: &str) {

        // self.write(format!("\t{} {}", escape_id(id), format_attributes(attrs)));
    }

    /// label needs to include the quotes
    pub fn add_node(&mut self, id: &str, attrs: &[(&str, &str)]) {
        let label = attrs
            .iter()
            .find_map(|&(key, val)| (key == "label").then_some(val))
            .unwrap_or_default();
        // self.write(format!("{}: \"{}\"", escape_id(id), escape_quote(label)));
        // self.write(format!("\t{} {}", escape_id(id), format_attributes(attrs)));
        self.nodes.push((id.to_string(), label.to_string()));
        self.wip_node_locations
            .insert(id.to_string(), NodeRef(vec![id.to_string()]));
    }
    pub fn add_invisible_node(&mut self, id: &str) {
        // self.add_node(id, &[("style", "invis"), ("label", ""), ("shape", "point")]);
    }

    /// The DOT syntax actually allows subgraphs as the edge's nodes but this doesn't support it yet.
    pub fn add_edge(&mut self, from: &str, to: &str, attrs: &[(&str, &str)]) {
        // self.edges
        //   .push((NodeRef(vec![from.to_owned()]), NodeRef(vec![to.to_owned()])));
        //self.write(format!("{} -> {}", escape_id(from), escape_id(to)));
        self.wip_edges.push((from.to_owned(), to.to_owned()));

        // self.add_edge_with_ports(from, None, to, None, attrs);
    }

    pub fn add_edge_with_ports(
        &mut self,
        from: &str,
        from_port: Option<&str>,
        to: &str,
        to_port: Option<&str>,
        attrs: &[(&str, &str)],
    ) {
        /*let from = if let Some(from_port) = from_port {
            format!("{}:{}", escape_id(from), escape_id(from_port))
        } else {
            escape_id(from).to_string()
        };
        let to = if let Some(to_port) = to_port {
            format!("{}:{}", escape_id(to), escape_id(to_port))
        } else {
            escape_id(to).to_string()
        };
        self.write(format!(
            "\t{} -> {} {}",
            &from,
            &to,
            format_attributes(attrs)
        ));*/
    }
}
