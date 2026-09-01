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
        // ★ The operand recurses through this same function, so a backgrounded command prints
        // identically to a standalone one with a wrapper above it -- the property that keeps a
        // golden captured either way agreeing.
        AstNode::Background(inner) => {
            out.push_str(&format!(
                "{}Background @[{},{})\n",
                indent(level),
                node.span.start,
                node.span.end
            ));
            render_node(inner, level + 1, out);
        }
        // ★ Stages are rendered by the SAME helper, not a parallel implementation. A pipeline is
        // composition: the stages are ordinary commands and must print identically to a standalone
        // one, or a golden captured from a pipeline would disagree with a golden captured alone.
        // Items recurse through this same function -- a sequence item IS an AstNode, unlike a
        // pipeline stage, so no helper is needed. The connector prints WITH the item it
        // introduces, matching how the AST pairs them.
        AstNode::Sequence(seq) => {
            out.push_str(&format!(
                "{}Sequence @[{},{})  {} items\n",
                indent(level),
                node.span.start,
                node.span.end,
                seq.rest.len() + 1
            ));
            render_node(&seq.first, level + 1, out);
            for (op, item) in &seq.rest {
                out.push_str(&format!("{}{:?}\n", indent(level + 1), op));
                render_node(item, level + 1, out);
            }
        }
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

/// Render a word's PARTS. ONE owner, shared by words and by assignment values -- an assignment
/// value is a Word like any other, and a second copy of this match would drift the way every
/// duplicated interpretation in this codebase has.
fn render_parts(parts: &[WordPart]) -> Vec<String> {
    parts
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
        .collect()
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
    // ASSIGNMENTS RENDER ONLY WHEN PRESENT, unlike `redirects: []` below.
    //
    // Every golden in golden.rs was captured before this field existed, so an
    // unconditional line would move ALL of them at once -- and a mass golden update is
    // exactly the change that hides one real diff among a hundred cosmetic ones.
    //
    // ⚠️ AND THE LINE ITSELF MATTERS. Without it `spine parse FOO=1 echo hi` printed only
    // `echo` and `hi`, so a correct parse read as a word being SILENTLY DROPPED. A debug
    // tool that under-reports is how two misreadings started this month.
    for a in &cmd.assignments {
        out.push_str(&format!(
            "{}Assign @[{},{})  {} = {}\n",
            indent(level + 1),
            a.span.start,
            a.span.end,
            a.node.name,
            render_parts(&a.node.value.parts).join(" ")
        ));
    }
    for word in &cmd.words {
        let rendered = render_parts(&word.node.parts);
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
            // INT-200: a target is a FILE or a STREAM, and the renderer says which. `2>&1`
            // showing as `1` would read as a filename in every golden captured from it.
            let t = match &r.node.target {
                super::ast::RedirectTarget::Stream(n) => format!("&{n}"),
                super::ast::RedirectTarget::File(w) => w
                    .parts
                    .iter()
                    .map(|p| match p {
                        WordPart::Literal { text, .. } => text.clone(),
                        WordPart::Variable { name, .. } => format!("${name}"),
                        WordPart::SpecialParam(s) => format!("{s:?}"),
                        WordPart::CommandSub(_) => "$(...)".to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(""),
            };
            out.push_str(&format!(
                "{}Redirect @[{},{})  {:?} -> {}\n",
                indent(level + 1),
                r.span.start,
                r.span.end,
                r.node.op,
                t
            ));
        }
    }
}
