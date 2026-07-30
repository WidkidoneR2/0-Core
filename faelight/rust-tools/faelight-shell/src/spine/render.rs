//! INT-169 spine: AST pretty-printer. Renders a parsed node as an indented tree with
//! spans, for the `spine parse` debug builtin and future introspection tooling. Display
//! only -- never executes anything.

use super::ast::{AstNode, Spanned, VariableSyntax, WordPart};

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
        AstNode::Command(cmd) => render_command(node.span, cmd, level, out),
        // ★ Stages are rendered by the SAME helper, not a parallel implementation. A pipeline is
        // composition: the stages are ordinary commands and must print identically to a standalone
        // one, or a golden captured from a pipeline would disagree with a golden captured alone.
        AstNode::Pipeline(p) => {
            out.push_str(&format!(
                "{}Pipeline @[{},{})  {} stages\n",
                indent(level),
                node.span.start,
                node.span.end,
                p.stages.len()
            ));
            for stage in &p.stages {
                render_command(stage.span, &stage.node, level + 1, out);
            }
        }
    }
}

fn render_command(
    span: super::ast::Span,
    cmd: &super::ast::Command,
    level: usize,
    out: &mut String,
) {
    out.push_str(&format!(
        "{}Command @[{},{})\n",
        indent(level),
        span.start,
        span.end
    ));
    for word in &cmd.words {
        let rendered: Vec<String> = word
            .node
            .parts
            .iter()
            .map(|p| match p {
                WordPart::Literal { text, .. } => format!("Literal {text:?}"),
                // Bare renders unchanged so existing goldens do not move; only the
                // braced form is annotated, because that is the fact that was missing.
                WordPart::Variable { name, syntax } => match syntax {
                    VariableSyntax::Bare => format!("Variable {name:?}"),
                    VariableSyntax::Braced => format!("Variable {name:?} (braced)"),
                },
                WordPart::SpecialParam(p) => format!("SpecialParam {p:?}"),
                // Shown, not erased, and not recursed into: this renderer has no
                // AST-traversal policy and inventing one here would be a second.
                WordPart::CommandSub(node) => {
                    format!("CommandSub @[{},{})", node.span.start, node.span.end)
                }
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
        // INT-200: redirects render for real now. The old line said "rendering TBD at roadmap
        // step 5" and printed only a count -- true when the field was permanently empty, stale
        // the moment the parser started filling it.
        for r in &cmd.redirects {
            let t: Vec<String> = r
                .node
                .target
                .parts
                .iter()
                .map(|p| match p {
                    WordPart::Literal { text, .. } => text.clone(),
                    WordPart::Variable { name, .. } => format!("${name}"),
                    WordPart::SpecialParam(s) => format!("{s:?}"),
                    WordPart::CommandSub(_) => "$(...)".to_string(),
                })
                .collect();
            out.push_str(&format!(
                "{}Redirect @[{},{})  {:?} -> {}\n",
                indent(level + 1),
                r.span.start,
                r.span.end,
                r.node.op,
                t.join("")
            ));
        }
    }
}
