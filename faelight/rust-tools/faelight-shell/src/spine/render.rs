//! INT-169 spine: AST pretty-printer. Renders a parsed node as an indented tree with
//! spans, for the `spine parse` debug builtin and future introspection tooling. Display
//! only -- never executes anything.

use super::ast::{AstNode, Spanned, WordPart};

/// Render a parsed node as an indented tree. Each line shows the construct and its span.
pub fn render(node: &Spanned<AstNode>) -> String {
    let mut out = String::new();
    render_node(node, 0, &mut out);
    out
}

fn indent(level: usize) -> String {
    "  ".repeat(level)
}

fn render_node(node: &Spanned<AstNode>, level: usize, out: &mut String) {
    match &node.node {
        AstNode::Command(cmd) => {
            out.push_str(&format!(
                "{}Command @[{},{})\n",
                indent(level),
                node.span.start,
                node.span.end
            ));
            for word in &cmd.words {
                let rendered: Vec<String> = word
                    .node
                    .parts
                    .iter()
                    .map(|p| match p {
                        WordPart::Literal(s) => format!("Literal {s:?}"),
                    })
                    .collect();
                out.push_str(&format!(
                    "{}Word @[{},{})  {}\n",
                    indent(level + 1),
                    word.span.start,
                    word.span.end,
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
