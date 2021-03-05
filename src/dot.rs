pub struct DotGraph {
    buffer: String,
}

fn format_attributes(attrs: &[(&str, &str)]) -> String {
    let attrs: Vec<_> = attrs.iter().map(|(a, b)| format!("{}={}", a, b)).collect();
    let attrs = attrs.join(", ");
    format!("[{}]", attrs)
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

fn html_escape(input: &str) -> String {
    input.replace('<', "&lt;").replace('>', "&gt;")
}

impl DotGraph {
    pub fn new(name: &str, options: &[(&str, &str)]) -> DotGraph {
        let mut dot = DotGraph {
            buffer: String::new(),
        };

        dot.write(format!("digraph {} {{", name));
        for (key, val) in options {
            dot.write(format!("\t{} = {};", key, val));
        }

        dot
    }

    pub fn edge_attributes(&mut self, attrs: &[(&str, &str)]) -> &mut Self {
        self.write(format!("\tedge {};", format_attributes(attrs)));
        self
    }

    pub fn node_attributes(&mut self, attrs: &[(&str, &str)]) -> &mut Self {
        self.write(format!("\tnode {};", format_attributes(attrs)));
        self
    }

    pub fn finish(mut self) -> String {
        self.write("}");
        self.buffer
    }

    /// label needs to include the quotes
    pub fn add_node(&mut self, id: &str, attrs: &[(&str, &str)]) {
        self.write(format!("\t{} {}", id, format_attributes(attrs)));
    }

    pub fn add_edge(
        &mut self,
        from: &str,
        from_port: Option<&str>,
        to: &str,
        to_port: Option<&str>,
        attrs: &[(&str, &str)],
    ) {
        let from = if let Some(from_port) = from_port {
            format!("{}:{}", from, from_port)
        } else {
            from.to_string()
        };
        let to = if let Some(to_port) = to_port {
            format!("{}:{}", to, to_port)
        } else {
            to.to_string()
        };
        self.write(format!("\t{} -> {} {}", from, to, format_attributes(attrs)));
    }

    fn write(&mut self, text: impl AsRef<str>) {
        self.buffer.push_str(text.as_ref());
        self.buffer.push('\n');
    }
}
