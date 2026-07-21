//! INT-169 spine: AST pretty-printer. Renders a parsed Node as an indented tree with
//! spans, for the `spine parse` debug builtin and future introspection tooling (the
//! debugger / "explain this parse" surfaces the RFC anticipates). Display only -- never
//! executes anything.

use super::ast::{Node, NodeKind, WordPart};

/// Render a parsed node as an indented tree. Each line shows the construct and its span.
pub fn render(node: &Node) -> String {
    let mut out = String::new();
    render_node(node, 0, &mut out);
    out
}

fn indent(level: usize) -> String {
    "  ".repeat(level)
}

fn render_node(node: &Node, level: usize, out: &mut String) {
    match &node.value {
        NodeKind::Command(cmd) => {
            out.push_str(&format!(
                "{}Command @[{},{})\n",
                indent(level),
                node.span.start,
                node.span.end
            ));
            for word in &cmd.words {
                // A Word has no span of its own yet (parser builds it from a token whose
                // span is folded into the Command span); we show its parts.
                let rendered: Vec<String> = word
                    .parts
                    .iter()
                    .map(|p| match p {
                        WordPart::Literal(s) => format!("Literal {s:?}"),
                    })
                    .collect();
                out.push_str(&format!(
                    "{}Word  {}\n",
                    indent(level + 1),
                    rendered.join(" ")
                ));
            }
            if cmd.redirects.is_empty() {
                out.push_str(&format!("{}redirects: []\n", indent(level + 1)));
            } else {
                out.push_str(&format!(
                    "{}redirects: {} (rendering TBD at roadmap step 5)\n",
                    indent(level + 1),
                    cmd.redirects.len()
                ));
            }
        }
    }
}
