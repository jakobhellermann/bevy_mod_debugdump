#![cfg_attr(not(feature = "render_graph"), allow(dead_code))]

use std::borrow::Cow;
pub struct DotGraph {
    buffer: String,
}

fn escape_quote(input: &str) -> Cow<'_, str> {
    if input.contains('"') {
        Cow::Owned(input.replace('"', "\\\""))
    } else {
        Cow::Borrowed(input)
    }
}

#[test]
fn escape_correctly() {
    assert_eq!(escape_id("a"), "\"a\"");
    assert_eq!(escape_id("contains \" quotes"), "\"contains \\\" quotes\"");
    assert_eq!(escape_id("RAW:<table>x</table>"), r#"<table>x</table>"#);

    let complex = "<Plugin>;build::{{closure}} → solve_constraint<FixedJoint, 2>";
    assert_eq!(escape_id(complex), format!("\"{complex}\""));
}

fn escape_id(input: &str) -> Cow<'_, str> {
    if let Some(raw) = input.strip_prefix("RAW:") {
        return raw.into();
    }

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

impl DotGraph {
    pub fn new(name: &str, kind: &str, attrs: &[(&str, &str)]) -> DotGraph {
        let mut dot = DotGraph {
            buffer: String::new(),
        };

        dot.write(format!("{} {} {{", kind, escape_id(name)));

        for (key, val) in attrs {
            dot.write(format!("\t{}={};", escape_id(key), escape_id(val)));
        }

        dot
    }

    pub fn digraph(name: &str, options: &[(&str, &str)]) -> DotGraph {
        DotGraph::new(name, "digraph", options)
    }
    pub fn subgraph(name: &str, options: &[(&str, &str)]) -> DotGraph {
        DotGraph::new(&format!("cluster{name}"), "subgraph", options)
    }

    #[allow(dead_code)]
    pub fn graph_attributes(mut self, attrs: &[(&str, &str)]) -> Self {
        self.write(format!("\tgraph {};", format_attributes(attrs)));
        self
    }

    pub fn edge_attributes(mut self, attrs: &[(&str, &str)]) -> Self {
        self.write(format!("\tedge {};", format_attributes(attrs)));
        self
    }

    pub fn node_attributes(mut self, attrs: &[(&str, &str)]) -> Self {
        self.write(format!("\tnode {};", format_attributes(attrs)));
        self
    }

    #[allow(unused)]
    pub fn same_rank<I, S>(&mut self, nodes: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.write_no_newline("{ rank = same;");
        for item in nodes {
            self.write(item);
            self.write("; ");
        }
        self.write("}");
    }

    pub fn finish(mut self) -> String {
        self.write("}");
        self.buffer
    }

    pub fn add_sub_graph(&mut self, graph: DotGraph) {
        let subgraph = graph.finish().replace('\n', "\n\t");
        self.write_no_newline("\t");
        self.write(subgraph);
    }

    /// label needs to include the quotes
    pub fn add_node(&mut self, id: &str, attrs: &[(&str, &str)]) {
        self.write(format!("\t{} {}", escape_id(id), format_attributes(attrs)));
    }
    pub fn add_invisible_node(&mut self, id: &str) {
        self.add_node(id, &[("style", "invis"), ("label", ""), ("shape", "point")]);
    }

    /// The DOT syntax actually allows subgraphs as the edge's nodes but this doesn't support it yet.
    pub fn add_edge(&mut self, from: &str, to: &str, attrs: &[(&str, &str)]) {
        self.add_edge_with_ports(from, None, to, None, attrs);
    }

    pub fn add_edge_with_ports(
        &mut self,
        from: &str,
        from_port: Option<&str>,
        to: &str,
        to_port: Option<&str>,
        attrs: &[(&str, &str)],
    ) {
        let from = if let Some(from_port) = from_port {
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
        ));
    }

    fn write_no_newline(&mut self, text: impl AsRef<str>) {
        self.buffer.push_str(text.as_ref());
    }

    fn write(&mut self, text: impl AsRef<str>) {
        self.buffer.push_str(text.as_ref());
        self.buffer.push('\n');
    }
}
