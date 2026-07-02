// faelight-shell — Pipeline Operator Tests
// INT-168 — Test Suite Foundation
//
// Tests for value.rs pipeline operators.
// Every operator tested with known input/output.

#[cfg(test)]
mod tests {
    use crate::value::{apply_pipeline, PipeOp, Value};
    use std::collections::HashMap;

    fn make_row(pairs: &[(&str, &str)]) -> HashMap<String, Value> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), Value::Text(v.to_string())))
            .collect()
    }

    fn make_num_row(pairs: &[(&str, f64)]) -> HashMap<String, Value> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), Value::Float(*v)))
            .collect()
    }

    fn make_table(rows: Vec<HashMap<String, Value>>) -> Value {
        Value::Table(rows)
    }

    // ── first operator ────────────────────────────────────────────────────────
    #[test]
    fn test_first_returns_n_rows() {
        let rows = (0..10)
            .map(|i| make_row(&[("n", &i.to_string())]))
            .collect();
        let result = apply_pipeline(make_table(rows), &[PipeOp::First(5)]);
        if let Value::Table(r) = result {
            assert_eq!(r.len(), 5);
        } else {
            panic!("expected Table");
        }
    }

    #[test]
    fn test_first_fewer_than_n() {
        let rows = (0..3).map(|i| make_row(&[("n", &i.to_string())])).collect();
        let result = apply_pipeline(make_table(rows), &[PipeOp::First(10)]);
        if let Value::Table(r) = result {
            assert_eq!(r.len(), 3);
        } else {
            panic!("expected Table");
        }
    }

    // ── last operator ─────────────────────────────────────────────────────────
    #[test]
    fn test_last_returns_n_rows() {
        let rows = (0..10)
            .map(|i| make_row(&[("n", &i.to_string())]))
            .collect();
        let result = apply_pipeline(make_table(rows), &[PipeOp::Last(3)]);
        if let Value::Table(r) = result {
            assert_eq!(r.len(), 3);
            assert_eq!(r[0].get("n").unwrap().as_text(), "7");
        } else {
            panic!("expected Table");
        }
    }

    // ── count operator ────────────────────────────────────────────────────────
    #[test]
    fn test_count_returns_int() {
        let rows = (0..7).map(|i| make_row(&[("n", &i.to_string())])).collect();
        let result = apply_pipeline(make_table(rows), &[PipeOp::Count]);
        if let Value::Int(n) = result {
            assert_eq!(n, 7);
        } else {
            panic!("expected Int");
        }
    }

    // ── where operator ────────────────────────────────────────────────────────
    #[test]
    fn test_where_equals_filter() {
        let rows = vec![
            make_row(&[("status", "pass")]),
            make_row(&[("status", "fail")]),
            make_row(&[("status", "pass")]),
        ];
        let op = PipeOp::Where {
            field: "status".into(),
            op: "==".into(),
            value: "pass".into(),
        };
        let result = apply_pipeline(make_table(rows), &[op]);
        if let Value::Table(r) = result {
            assert_eq!(r.len(), 2);
        } else {
            panic!("expected Table");
        }
    }

    #[test]
    fn test_where_contains_filter() {
        let rows = vec![
            make_row(&[("msg", "feat: add login")]),
            make_row(&[("msg", "fix: crash on startup")]),
            make_row(&[("msg", "feat: improve UI")]),
        ];
        let op = PipeOp::Where {
            field: "msg".into(),
            op: "contains".into(),
            value: "feat".into(),
        };
        let result = apply_pipeline(make_table(rows), &[op]);
        if let Value::Table(r) = result {
            assert_eq!(r.len(), 2);
        } else {
            panic!("expected Table");
        }
    }

    // ── sort operator ─────────────────────────────────────────────────────────
    #[test]
    fn test_sort_asc() {
        let rows = vec![
            make_row(&[("name", "charlie")]),
            make_row(&[("name", "alice")]),
            make_row(&[("name", "bob")]),
        ];
        let op = PipeOp::Sort {
            field: "name".into(),
            desc: false,
        };
        let result = apply_pipeline(make_table(rows), &[op]);
        if let Value::Table(r) = result {
            assert_eq!(r[0].get("name").unwrap().as_text(), "alice");
            assert_eq!(r[2].get("name").unwrap().as_text(), "charlie");
        } else {
            panic!("expected Table");
        }
    }

    // ── unique operator ───────────────────────────────────────────────────────
    #[test]
    fn test_unique_deduplicates() {
        let rows = vec![
            make_row(&[("domain", "shell")]),
            make_row(&[("domain", "core")]),
            make_row(&[("domain", "shell")]),
        ];
        let op = PipeOp::Unique {
            field: "domain".into(),
        };
        let result = apply_pipeline(make_table(rows), &[op]);
        if let Value::Table(r) = result {
            assert_eq!(r.len(), 2);
        } else {
            panic!("expected Table");
        }
    }

    // ── reduce operator ───────────────────────────────────────────────────────
    #[test]
    fn test_reduce_sum() {
        let rows = vec![
            make_num_row(&[("cpu", 10.0)]),
            make_num_row(&[("cpu", 20.0)]),
            make_num_row(&[("cpu", 30.0)]),
        ];
        let op = PipeOp::Reduce {
            expr: "sum cpu".into(),
        };
        let result = apply_pipeline(make_table(rows), &[op]);
        if let Value::Float(n) = result {
            assert!((n - 60.0).abs() < 0.001);
        } else {
            panic!("expected Float, got {:?}", result);
        }
    }

    // ── to-text operator ──────────────────────────────────────────────────────
    #[test]
    fn test_to_text_produces_string() {
        let rows = vec![make_row(&[("name", "core"), ("status", "ok")])];
        let result = apply_pipeline(make_table(rows), &[PipeOp::ToText]);
        match result {
            Value::Text(s) => assert!(!s.is_empty()),
            _ => panic!("expected Text"),
        }
    }

    // ── pipeline chaining ─────────────────────────────────────────────────────
    #[test]
    fn test_chained_first_count() {
        let rows = (0..20)
            .map(|i| make_row(&[("n", &i.to_string())]))
            .collect();
        let ops = vec![PipeOp::First(5), PipeOp::Count];
        let result = apply_pipeline(make_table(rows), &ops);
        if let Value::Int(n) = result {
            assert_eq!(n, 5);
        } else {
            panic!("expected Int");
        }
    }
}
