// faelight-shell — Value type system
// Phase 2: structured data pipeline
// "Not text streams. Structured wisdom."

use colored::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Value {
    Text(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Row(HashMap<String, Value>),
    Table(Vec<HashMap<String, Value>>),
    Nothing,
}

impl Value {
    pub fn as_text(&self) -> String {
        match self {
            Value::Text(s) => s.clone(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => format!("{:.2}", f),
            Value::Bool(b) => b.to_string(),
            Value::Nothing => "".to_string(),
            Value::Row(_) => "[row]".to_string(),
            Value::Table(t) => format!("[table: {} rows]", t.len()),
        }
    }

    #[allow(dead_code)]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            Value::Float(f) => Some(*f as i64),
            Value::Text(s) => s.parse().ok(),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Text(s) => !s.is_empty(),
            Value::Nothing => false,
            _ => true,
        }
    }

    /// Render as a formatted table for display
    pub fn render(&self) -> String {
        match self {
            Value::Table(rows) => render_table(rows),
            Value::Row(row) => render_row(row),
            Value::Text(s) => format!("  {}", s),
            Value::Int(i) => format!("  {}", i.to_string().bright_white()),
            Value::Float(f) => format!("  {:.2}", f),
            Value::Bool(b) => format!(
                "  {}",
                if *b {
                    "true".bright_green()
                } else {
                    "false".bright_red()
                }
            ),
            Value::Nothing => String::new(),
        }
    }
}

pub fn render_table(rows: &[HashMap<String, Value>]) -> String {
    if rows.is_empty() {
        return format!("  {}", "No results.".dimmed());
    }

    // Collect column names -- name/line first, then sorted rest, hide size_bytes
    let raw_keys: Vec<String> = rows[0].keys().cloned().collect();
    let priority = [
        "name", "line", "n", "size", "type", "kind", "domain", "action",
    ];
    let mut headers: Vec<String> = priority
        .iter()
        .filter(|p| raw_keys.contains(&p.to_string()))
        .map(|p| p.to_string())
        .collect();
    for k in &raw_keys {
        if k == "size_bytes" {
            continue;
        } // internal column, skip display
        if !headers.contains(k) {
            headers.push(k.clone());
        }
    }

    // Calculate column widths
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, h) in headers.iter().enumerate() {
            let val = row.get(h).map(|v| v.as_text()).unwrap_or_default();
            if i < widths.len() {
                widths[i] = widths[i].max(val.len().min(40));
            }
        }
    }

    let mut out = String::new();
    out.push('\n');

    // Header
    out.push_str("  ");
    for (i, h) in headers.iter().enumerate() {
        out.push_str(&format!(
            "{:<width$}  ",
            h.bright_white().bold(),
            width = widths[i]
        ));
    }
    out.push('\n');

    // Separator
    out.push_str("  ");
    for w in &widths {
        out.push_str(&format!("{}  ", "─".repeat(*w).dimmed()));
    }
    out.push('\n');

    // Rows
    for row in rows {
        out.push_str("  ");
        for (i, h) in headers.iter().enumerate() {
            if i < widths.len() {
                let val = row.get(h).map(|v| v.as_text()).unwrap_or_default();
                let truncated = if val.chars().count() > 40 {
                    format!("{}...", val.chars().take(37).collect::<String>())
                } else {
                    val
                };
                out.push_str(&format!(
                    "{:<width$}  ",
                    truncated.dimmed(),
                    width = widths[i]
                ));
            }
        }
        out.push('\n');
    }

    out
}

fn render_row(row: &HashMap<String, Value>) -> String {
    let mut out = String::new();
    for (k, v) in row {
        out.push_str(&format!(
            "  {:<20} {}\n",
            k.bright_white(),
            v.as_text().dimmed()
        ));
    }
    out
}

/// Pipeline — apply data commands to a Value
pub fn apply_pipeline(value: Value, ops: &[PipeOp]) -> Value {
    let mut current = value;
    for op in ops {
        current = apply_op(current, op);
    }
    current
}
/// INT-322 Phase 5: per-stage stats for --explain flag
pub struct PipelineStageStats {
    pub label: String,
    pub row_count: usize,
    pub duration_ms: u128,
}

pub fn apply_pipeline_with_stats(
    value: Value,
    ops: &[PipeOp],
    stage_labels: &[String],
) -> (Value, Vec<PipelineStageStats>) {
    let mut current = value;
    let mut stats: Vec<PipelineStageStats> = Vec::new();
    for (i, op) in ops.iter().enumerate() {
        let start = std::time::Instant::now();
        current = apply_op(current, op);
        let duration_ms = start.elapsed().as_millis();
        let row_count = match &current {
            Value::Table(rows) => rows.len(),
            _ => 1,
        };
        let label = stage_labels
            .get(i)
            .cloned()
            .unwrap_or_else(|| format!("stage {}", i + 1));
        stats.push(PipelineStageStats {
            label,
            row_count,
            duration_ms,
        });
    }
    (current, stats)
}

// ── Phase 2: Schema System — INT-162 ─────────────────────────────────────────
// Typed schemas guarantee consistent column names across pipeline operators.
// Each schema implements to_row() → HashMap<String, Value> for pipeline compat.

/// Schema for `ps` output
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProcessRow {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub memory: f32,
    pub status: String,
}

#[allow(dead_code)]
impl ProcessRow {
    pub fn to_row(&self) -> HashMap<String, Value> {
        let mut m = HashMap::new();
        m.insert("pid".into(), Value::Int(self.pid as i64));
        m.insert("name".into(), Value::Text(self.name.clone()));
        m.insert("cpu".into(), Value::Float(self.cpu as f64));
        m.insert("memory".into(), Value::Float(self.memory as f64));
        m.insert("status".into(), Value::Text(self.status.clone()));
        m
    }
    pub fn columns() -> &'static [&'static str] {
        &["pid", "name", "cpu", "memory", "status"]
    }
}

/// Schema for `gc` output
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CommitRow {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub date: String,
    pub domain: String,
}

#[allow(dead_code)]
impl CommitRow {
    pub fn to_row(&self) -> HashMap<String, Value> {
        let mut m = HashMap::new();
        m.insert("hash".into(), Value::Text(self.hash.clone()));
        m.insert("message".into(), Value::Text(self.message.clone()));
        m.insert("author".into(), Value::Text(self.author.clone()));
        m.insert("date".into(), Value::Text(self.date.clone()));
        m.insert("domain".into(), Value::Text(self.domain.clone()));
        m
    }
    pub fn columns() -> &'static [&'static str] {
        &["hash", "message", "author", "date", "domain"]
    }
}

/// Schema for health check output
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HealthRow {
    pub check: String,
    pub status: String,
    pub message: String,
}

#[allow(dead_code)]
impl HealthRow {
    pub fn to_row(&self) -> HashMap<String, Value> {
        let mut m = HashMap::new();
        m.insert("check".into(), Value::Text(self.check.clone()));
        m.insert("status".into(), Value::Text(self.status.clone()));
        m.insert("message".into(), Value::Text(self.message.clone()));
        m
    }
    pub fn columns() -> &'static [&'static str] {
        &["check", "status", "message"]
    }
}

#[derive(Clone)]
pub enum PipeOp {
    Where {
        field: String,
        op: String,
        value: String,
    },
    Select {
        fields: Vec<String>,
    },
    Sort {
        field: String,
        desc: bool,
    },
    First(usize),
    Last(usize),
    Count,
    Get(String),
    Watch {
        interval: u64,
    },
    Join {
        table: String,
        on: String,
    },
    JoinData {
        rows: Vec<std::collections::HashMap<String, Value>>,
        on: String,
    },
    // pipe to external unix command
    #[allow(dead_code)]
    External(String),
    Group {
        field: String,
    },
    // Phase 3 — INT-162 pipeline operators
    Map {
        expr: String,
    },
    Reduce {
        expr: String,
    },
    Unique {
        field: String,
    },
    Flatten,
    ToText,
    Skip(usize),
    AsJson,
    UniqueAll, // deduplicate rows by all fields combined
}

fn apply_op(value: Value, op: &PipeOp) -> Value {
    match (value, op) {
        (
            Value::Table(rows),
            PipeOp::Where {
                field,
                op,
                value: filter_val,
            },
        ) => {
            let filtered: Vec<_> = rows
                .into_iter()
                .filter(|row| {
                    // _any = search all fields
                    let cell = if field == "_any" {
                        row.values()
                            .map(|v| v.as_text())
                            .collect::<Vec<_>>()
                            .join(" ")
                    } else {
                        row.get(field).map(|v| v.as_text()).unwrap_or_default()
                    };
                    match op.as_str() {
                        "==" | "=" => cell == *filter_val,
                        "!=" => cell != *filter_val,
                        ">" => cell.parse::<f64>().ok() > filter_val.parse::<f64>().ok(),
                        "<" => cell.parse::<f64>().ok() < filter_val.parse::<f64>().ok(),
                        ">=" => cell.parse::<f64>().ok() >= filter_val.parse::<f64>().ok(),
                        "<=" => cell.parse::<f64>().ok() <= filter_val.parse::<f64>().ok(),
                        "contains" => cell.to_lowercase().contains(&filter_val.to_lowercase()),
                        _ => false,
                    }
                })
                .collect();
            Value::Table(filtered)
        }
        (Value::Table(rows), PipeOp::Select { fields }) => {
            let selected: Vec<_> = rows
                .into_iter()
                .map(|row| {
                    fields
                        .iter()
                        .filter_map(|f| row.get(f).map(|v| (f.clone(), v.clone())))
                        .collect()
                })
                .collect();
            Value::Table(selected)
        }
        (Value::Table(mut rows), PipeOp::Sort { field, desc }) => {
            rows.sort_by(|a, b| {
                let effective_field = if field == "_first" {
                    a.keys().next().cloned().unwrap_or_default()
                } else {
                    field.clone()
                };
                let av = a
                    .get(&effective_field)
                    .map(|v| v.as_text())
                    .unwrap_or_default();
                let bv = b
                    .get(&effective_field)
                    .map(|v| v.as_text())
                    .unwrap_or_default();
                // Try numeric sort first
                let cmp = if let (Ok(an), Ok(bn)) = (av.parse::<f64>(), bv.parse::<f64>()) {
                    an.partial_cmp(&bn).unwrap_or(std::cmp::Ordering::Equal)
                } else {
                    av.cmp(&bv)
                };
                if *desc {
                    cmp.reverse()
                } else {
                    cmp
                }
            });
            Value::Table(rows)
        }
        (Value::Table(rows), PipeOp::First(n)) => Value::Table(rows.into_iter().take(*n).collect()),
        (Value::Table(rows), PipeOp::Last(n)) => {
            let len = rows.len();
            Value::Table(rows.into_iter().skip(len.saturating_sub(*n)).collect())
        }
        (Value::Table(rows), PipeOp::Count) => Value::Int(rows.len() as i64),
        (Value::Table(rows), PipeOp::Get(field)) => {
            if rows.len() == 1 {
                rows[0].get(field).cloned().unwrap_or(Value::Nothing)
            } else {
                Value::Table(
                    rows.into_iter()
                        .filter_map(|row| {
                            row.get(field).map(|v| {
                                let mut m = HashMap::new();
                                m.insert(field.clone(), v.clone());
                                m
                            })
                        })
                        .collect(),
                )
            }
        }
        // Group — aggregate rows by field value, count each group
        (Value::Table(rows), PipeOp::Group { field }) => {
            use std::collections::HashMap as HM;
            let mut counts: std::collections::BTreeMap<String, usize> =
                std::collections::BTreeMap::new();
            for row in &rows {
                let key = row.get(field).map(|v| v.as_text()).unwrap_or_default();
                *counts.entry(key).or_insert(0) += 1;
            }
            let total = rows.len();
            let grouped: Vec<HM<String, Value>> = counts
                .into_iter()
                .map(|(key, count)| {
                    let pct = format!("{:.1}%", (count as f64 / total as f64) * 100.0);
                    let mut row = HM::new();
                    row.insert(field.clone(), Value::Text(key));
                    row.insert("count".to_string(), Value::Int(count as i64));
                    row.insert("pct".to_string(), Value::Text(pct));
                    row
                })
                .collect();
            Value::Table(grouped)
        }
        // JoinData — pre-resolved join (table already fetched by main.rs)
        (
            Value::Table(left_rows),
            PipeOp::JoinData {
                rows: right_rows,
                on,
            },
        ) => {
            let mut result = vec![];
            for left in left_rows {
                let left_key = left.get(on).map(|v| v.as_text()).unwrap_or_default();
                // Find matching right rows
                let matches: Vec<_> = right_rows
                    .iter()
                    .filter(|r| r.get(on).map(|v| v.as_text()).unwrap_or_default() == left_key)
                    .collect();
                if matches.is_empty() {
                    // Left join — include left row even with no match
                    result.push(left.clone());
                } else {
                    for right in matches {
                        let mut merged = left.clone();
                        for (k, v) in right {
                            if k != on {
                                // Prefix right-side columns to avoid collision
                                merged.insert(format!("r_{}", k), v.clone());
                            }
                        }
                        result.push(merged);
                    }
                }
            }
            Value::Table(result)
        }
        // Phase 3 — map: transform each row's field with a simple expression
        (Value::Table(rows), PipeOp::Map { expr }) => {
            let parts: Vec<&str> = expr.splitn(3, ' ').collect();
            let mapped: Vec<_> = rows
                .into_iter()
                .map(|mut row| {
                    // Supported: "field * 2", "field + N", "field - N"
                    if parts.len() == 3 {
                        let field = parts[0];
                        let op = parts[1];
                        let rhs: f64 = parts[2].parse().unwrap_or(0.0);
                        if let Some(val) = row.get(field).and_then(|v| match v {
                            Value::Float(f) => Some(*f),
                            Value::Int(i) => Some(*i as f64),
                            Value::Text(s) => s.parse::<f64>().ok(),
                            _ => None,
                        }) {
                            let result = match op {
                                "*" => val * rhs,
                                "+" => val + rhs,
                                "-" => val - rhs,
                                "/" => {
                                    if rhs != 0.0 {
                                        val / rhs
                                    } else {
                                        val
                                    }
                                }
                                _ => val,
                            };
                            row.insert(field.to_string(), Value::Float(result));
                        }
                    }
                    row
                })
                .collect();
            Value::Table(mapped)
        }
        // Phase 3 — reduce: aggregate a numeric field to a single value
        (Value::Table(rows), PipeOp::Reduce { expr }) => {
            let parts: Vec<&str> = expr.splitn(2, ' ').collect();
            let (agg, field) = if parts.len() == 2 {
                (parts[0], parts[1])
            } else {
                return Value::Nothing;
            };
            let nums: Vec<f64> = rows
                .iter()
                .filter_map(|r| match r.get(field)? {
                    Value::Float(f) => Some(*f),
                    Value::Int(i) => Some(*i as f64),
                    Value::Text(s) => s.parse::<f64>().ok(),
                    _ => None,
                })
                .collect();
            if nums.is_empty() {
                return Value::Nothing;
            }
            let result = match agg {
                "sum" => nums.iter().sum(),
                "avg" => nums.iter().sum::<f64>() / nums.len() as f64,
                "min" => nums.iter().cloned().fold(f64::INFINITY, f64::min),
                "max" => nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
                _ => return Value::Nothing,
            };
            Value::Float(result)
        }
        // Phase 3 — unique: deduplicate rows by field value
        (Value::Table(rows), PipeOp::Unique { field }) => {
            let mut seen = std::collections::HashSet::new();
            let unique: Vec<_> = rows
                .into_iter()
                .filter(|row| {
                    let key = row.get(field).map(|v| v.as_text()).unwrap_or_default();
                    seen.insert(key)
                })
                .collect();
            Value::Table(unique)
        }
        // Phase 3 — flatten: expand Table-of-Tables into a single Table
        (Value::Table(rows), PipeOp::Flatten) => {
            let mut flat = vec![];
            for row in rows {
                let mut is_nested = false;
                for v in row.values() {
                    if let Value::Table(inner) = v {
                        flat.extend(inner.clone());
                        is_nested = true;
                        break;
                    }
                }
                if !is_nested {
                    flat.push(row);
                }
            }
            Value::Table(flat)
        }
        // Phase 3 — to-text: serialize Table to plain text (external boundary)
        (Value::Table(rows), PipeOp::ToText) => {
            if rows.is_empty() {
                return Value::Text(String::new());
            }
            let headers: Vec<String> = rows[0].keys().cloned().collect();
            let mut lines = vec![headers.join("\t")];
            for row in &rows {
                let line: Vec<String> = headers
                    .iter()
                    .map(|h| row.get(h).map(|v| v.as_text()).unwrap_or_default())
                    .collect();
                lines.push(line.join("\t"));
            }
            Value::Text(lines.join("\n"))
        }
        (Value::Table(rows), PipeOp::Skip(n)) => Value::Table(rows.into_iter().skip(*n).collect()),
        (Value::Table(rows), PipeOp::AsJson) => {
            let mut out = vec![];
            for row in &rows {
                let mut obj = String::from("{");
                let pairs: Vec<String> = row
                    .iter()
                    .map(|(k, v)| format!("\"{}\":\"{}\"", k, v.as_text()))
                    .collect();
                obj.push_str(&pairs.join(","));
                obj.push('}');
                out.push(obj);
            }
            Value::Text(format!("[{}]", out.join(",")))
        }
        (Value::Table(rows), PipeOp::UniqueAll) => {
            let mut seen = std::collections::HashSet::new();
            let unique: Vec<_> = rows
                .into_iter()
                .filter(|row| {
                    let mut keys: Vec<String> = row
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v.as_text()))
                        .collect();
                    keys.sort();
                    seen.insert(keys.join("|"))
                })
                .collect();
            Value::Table(unique)
        }
        (v, _) => v, // passthrough for non-table values
    }
}

/// The forest's pipeline vocabulary -- the verbs that make a `|` mean DATA FLOW rather than a
/// process pipe. INT-200: extracted so the spine can ask whether a pipeline is its own to execute.
///
/// ★ WHY A CONST AND NOT A SECOND MATCH: `parse_pipe_op` below dispatches on structural patterns
/// (`["sort", "by", field]`), which cannot be reduced to a name lookup without losing arity. So the
/// name list cannot literally share code with it -- but it CAN share a proof, and
/// `value_verbs_match_the_parser` below is that proof. Add a verb to one and the test fails.
///
/// ⚠️ THE SPINE MUST NOT KEEP ITS OWN COPY. A second vocabulary is the two-owners failure INT-193
/// existed to end; it would drift the first time a verb is added here.
/// The SOURCES a value pipeline can begin with. INT-201 (2026-08-06).
///
/// ★ THE VERBS ALONE COULD NOT ANSWER THE OWNERSHIP QUESTION, and that gap was a live bug. A
/// pipeline was called "forest" whenever any later stage named a value verb -- a statement about a
/// WORD. But a language is identified by where it STARTS, not by a word appearing in the middle, so
/// `echo a | sort -k1 -rn` was claimed by the query language, refused by the spine, and then refused
/// again by legacy once the inline pipeline executor was deleted.
///
/// ⚠️ THE VERB LIST AND THE SHELL SHARE A NAMESPACE. Every word added here or to VALUE_VERBS is a
/// word that stops being available as a pipeline stage. `join`, `watch`, `get`, `filter` and `sort`
/// are all real programs. That is the cost of a query language that looks like shell, and it is
/// paid by whoever adds the next verb.
///
/// ⚠️ RESIDUAL AMBIGUITY, stated rather than hidden: `find`, `ps`, `list`, `files` and `db` are BOTH
/// sources and real commands, so `find . -name x | sort` is still read as a query. This list fixes
/// pipelines that begin with an ORDINARY command; it does not disambiguate the overlap.
pub const VALUE_SOURCES: &[&str] = &[
    "from",
    "list",
    "find",
    "db",
    "intents",
    "deploys",
    "friday",
    "ps",
    "processes",
    "files",
    "tools",
    "events",
];

/// Does this word begin a value PIPELINE? Pure, and first-word only.
pub fn is_value_source(first: &str) -> bool {
    VALUE_SOURCES.contains(&first)
}

pub const VALUE_VERBS: &[&str] = &[
    "where", "select", "sort", "first", "last", "count", "take", "skip", "unique", "as", "filter",
    "get", "watch", "group", "join", "map", "reduce", "flatten", "to-text",
];

/// Does this word begin a VALUE operation rather than a command? Pure, and first-word only.
///
/// ★ FIRST WORD IS ENOUGH FOR THE OWNERSHIP QUESTION even though `parse_pipe_op` needs the whole
/// segment to build the op. "Is this stage mine?" and "what exactly does it do?" are different
/// questions, and the cheaper one is the one the router needs.
pub fn is_value_verb(first: &str) -> bool {
    VALUE_VERBS.contains(&first)
}

/// Parse a pipeline string like "where score < 70 | sort score | first 5"
pub fn parse_pipeline(s: &str) -> Vec<PipeOp> {
    s.split('|')
        .skip(1) // first segment is the command itself
        .map(|seg| seg.trim())
        .filter_map(parse_pipe_op)
        .collect()
}

fn parse_pipe_op(s: &str) -> Option<PipeOp> {
    let parts: Vec<&str> = s.splitn(4, ' ').collect();
    match parts.as_slice() {
        ["where", field, op, val] => Some(PipeOp::Where {
            field: field.to_string(),
            op: op.to_string(),
            value: val.trim_matches('"').to_string(),
        }),
        ["select", rest @ ..] => Some(PipeOp::Select {
            fields: rest
                .join(" ")
                .split([',', ' '])
                .map(|f| f.trim().to_string())
                .filter(|f| !f.is_empty())
                .collect(),
        }),
        ["sort", field] => Some(PipeOp::Sort {
            field: field.to_string(),
            desc: false,
        }),
        ["sort", field, "desc"] => Some(PipeOp::Sort {
            field: field.to_string(),
            desc: true,
        }),
        ["first", n] => n.parse().ok().map(PipeOp::First),
        ["last", n] => n.parse().ok().map(PipeOp::Last),
        ["count"] => Some(PipeOp::Count),
        // INT-265: human-readable aliases
        ["take", n] => n.parse().ok().map(PipeOp::First),
        ["skip", n] => n.parse().ok().map(PipeOp::Skip),
        ["unique"] => Some(PipeOp::UniqueAll),
        ["as", "json"] => Some(PipeOp::AsJson),
        // filter contains "x" → where any-field contains x
        ["filter", "contains", val] => Some(PipeOp::Where {
            field: "_any".to_string(),
            op: "contains".to_string(),
            value: val.trim_matches('"').to_string(),
        }),
        // filter <col> <op> <val>
        ["filter", field, op, val] => Some(PipeOp::Where {
            field: field.to_string(),
            op: op.to_string(),
            value: val.trim_matches('"').to_string(),
        }),
        // sort by <col> descending
        ["sort", "by", field, "descending"] => Some(PipeOp::Sort {
            field: field.to_string(),
            desc: true,
        }),
        // sort by <col>
        ["sort", "by", field] => Some(PipeOp::Sort {
            field: field.to_string(),
            desc: false,
        }),
        // sort (no field -- sort by first column, handled at runtime)
        ["sort"] => Some(PipeOp::Sort {
            field: "_first".to_string(),
            desc: false,
        }),
        ["get", field] => Some(PipeOp::Get(field.to_string())),
        ["watch"] => Some(PipeOp::Watch { interval: 2 }),
        ["watch", n] => n.parse().ok().map(|i| PipeOp::Watch { interval: i }),
        // group <field>
        ["group", field] => Some(PipeOp::Group {
            field: field.to_string(),
        }),
        // join <table> on <field>
        ["join", table, "on", field] => Some(PipeOp::Join {
            table: table.to_string(),
            on: field.to_string(),
        }),
        // Phase 3 — map <field> <op> <val>
        ["map", ..] => Some(PipeOp::Map {
            expr: s.trim_start_matches("map").trim().to_string(),
        }),
        // Phase 3 — reduce <agg> <field>
        ["reduce", ..] => Some(PipeOp::Reduce {
            expr: s.trim_start_matches("reduce").trim().to_string(),
        }),
        // Phase 3 — unique <field>
        ["unique", field] => Some(PipeOp::Unique {
            field: field.to_string(),
        }),
        // Phase 3 — flatten
        ["flatten"] => Some(PipeOp::Flatten),
        // Phase 3 — to-text
        ["to-text"] => Some(PipeOp::ToText),
        _ => {
            // Unknown pipe op — try as external command
            let cmd = s.trim().to_string();
            if cmd.is_empty() {
                return None;
            }
            // Only treat as external if it looks like a binary name
            let first = cmd.split_whitespace().next().unwrap_or("");
            if first
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '/')
            {
                Some(PipeOp::External(cmd))
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod pipeline_vocabulary_tests {
    use super::*;

    /// ★ THE DRIFT GUARD, and the reason the const is allowed to exist. Each sample is run through
    /// BOTH the real parser and the predicate; they must agree. A verb added to `parse_pipe_op`
    /// without being added to `VALUE_VERBS` fails here, because the parser will build a real op
    /// while the predicate still calls the stage external.
    ///
    /// ⚠️ This asserts AGREEMENT, not correctness of either side alone -- which is the honest thing
    /// a cross-check can promise.
    #[test]
    fn value_verbs_match_the_parser() {
        for sample in [
            "where score < 70",
            "select name score",
            "sort score",
            "sort by score descending",
            "first 5",
            "last 3",
            "count",
            "take 2",
            "skip 2",
            "unique",
            "as json",
            "filter contains x",
            "get name",
            "watch",
            "group domain",
            "join ports on pid",
            "map score * 2",
            "reduce sum score",
            "flatten",
            "to-text",
        ] {
            let first = sample.split_whitespace().next().unwrap();
            let parsed = parse_pipe_op(sample);
            let is_external = matches!(parsed, Some(PipeOp::External(_)));
            assert!(
                is_value_verb(first),
                "{sample:?} parses as a value op but {first:?} is missing from VALUE_VERBS"
            );
            assert!(
                !is_external,
                "{sample:?} was treated as an external command"
            );
        }
    }

    /// The inverse, and the one that keeps the predicate from swallowing real commands. These are
    /// ordinary programs and must stay external, or the spine would decline pipelines it owns.
    #[test]
    fn ordinary_commands_are_not_value_verbs() {
        for sample in ["grep foo", "wc -l", "head -5", "sort_by_hand", "counter"] {
            let first = sample.split_whitespace().next().unwrap();
            assert!(
                !is_value_verb(first),
                "{first:?} must not be treated as a forest verb"
            );
        }
    }
}
