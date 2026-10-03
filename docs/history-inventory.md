# shell_history inventory (INT-191 G1)

GENERATED. Do not edit by hand -- run
`python3 zero/shell/novashell/generate-history-inventory.py`.

- writers: **4**
- readers: **80**
- untyped (multi-line statements a single line cannot classify): **24**
- matched but NOT consumers: **8**

## Writers

The gate asks whether every history write has a single, well-defined owner.

- `shell/novashell/src/db.rs:331` -- "INSERT INTO shell_history (command, timestamp, cwd, intent_id) VALUES (?1, ?2, ?3, ?4)",
- `shell/novashell/src/db.rs:367` -- "UPDATE shell_history SET exit_code = ?1, duration_ms = ?2 WHERE id = ?3",
- `shell/novashell/src/engine.rs:1821` -- "INSERT INTO shell_history (command, timestamp) VALUES (?1, ?2)",
- `shell/novashell/src/main.rs:2997` -- "INSERT INTO shell_history (command, timestamp) VALUES (?1, strftime('%s','now'))",

## Matched but not consumers

Each was read and judged rather than excluded by a pattern.

- `shell/novashell/src/commands/mod.rs:14687` -- fsh doctor writability probe -- inserts and deletes its own row
- `shell/novashell/src/commands/mod.rs:14692` -- fsh doctor writability probe -- inserts and deletes its own row
- `shell/novashell/src/commands/mod.rs:16680` -- retention pruning, not recording
- `shell/novashell/src/commands/mod.rs:16688` -- retention pruning, not recording
- `shell/novashell/src/db.rs:213` -- trigger definition (shell_history_audit)
- `shell/novashell/src/db.rs:218` -- writes the AUDIT table, not history
- `shell/novashell/src/db.rs:222` -- immutability guard on the audit table
- `shell/novashell/src/db.rs:227` -- immutability guard on the audit table

## Readers, by file

### engine/src/domains/db/mod.rs (1)

- `177` [UNTYPED] "shell_history",

### engine/src/domains/friday/mod.rs (13)

- `720` [READ] "SELECT COUNT(*) FROM shell_history h1
- `723` [READ] SELECT 1 FROM shell_history h2
- `739` [UNTYPED] VALUES ('cicomplete runs', '[\"deploy\", \"workflow\"]', 'deploy tool', 'success', ?1, ?2, ?3, 'shel
- `751` [READ] "SELECT COUNT(*) FROM shell_history h1
- `754` [READ] SELECT 1 FROM shell_history h2
- `770` [UNTYPED] VALUES ('deploy completes', '[\"commit\", \"workflow\"]', 'fg commit', 'success', ?1, ?2, ?3, 'shell
- `782` [READ] "SELECT command, COUNT(*) as cnt FROM shell_history
- `800` [UNTYPED] VALUES ('frequent command', '[\"habit\"]', ?1, 'observed', ?2, 0.8, ?3, 'shell_history')",
- `817` [READ] db.query_row("SELECT COUNT(*) FROM shell_history", [], |r| r
- `913` [READ] "SELECT COUNT(*) FROM shell_history WHERE timestamp > ?1",
- `1193` [READ] "SELECT command FROM shell_history WHERE command LIKE 'python3 /tmp/%'
- `1439` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE ?1 AND timestamp > ?2",
- `1594` [READ] .query_row("SELECT COUNT(*) FROM shell_history", [], |r| r.get(0))

### engine/src/domains/friday/planning.rs (4)

- `447` [READ] "SELECT COUNT(*) FROM shell_history \
- `489` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'cistart%' AND timestamp > ?1",
- `496` [READ] "SELECT COUNT(*) FROM shell_history WHERE timestamp > ?1",
- `812` [READ] "SELECT command FROM shell_history \

### engine/src/domains/friday/reasoning.rs (2)

- `105` [READ] "SELECT COUNT(*) FROM shell_history WHERE timestamp > ?1 AND (command LIKE 'cistart%' OR command LIK
- `111` [READ] "SELECT COUNT(*) FROM shell_history WHERE timestamp > ?1",

### engine/src/domains/friday_arch/mod.rs (3)

- `1106` [READ] "SELECT COUNT(*) FROM shell_history
- `1138` [READ] "SELECT COUNT(*) FROM shell_history
- `1181` [READ] "SELECT COUNT(*) FROM shell_history WHERE command = ?1",

### engine/src/domains/predict/mod.rs (2)

- `1428` [READ] "SELECT command, COUNT(*) as freq FROM shell_history
- `1476` [READ] .query_row("SELECT COUNT(*) FROM shell_history", [], |r| r.get(0))

### shell/novashell/src/commands/mod.rs (53)

- `805` [READ] "SELECT command FROM shell_history \
- `855` [READ] "SELECT DISTINCT command FROM shell_history \
- `1902` [READ] "SELECT command FROM shell_history ORDER BY id DESC LIMIT 20"
- `2049` [READ] .prepare("SELECT id, command FROM shell_history ORDER BY id DESC LIMIT ?1")
- `2425` [READ] "SELECT audit_id, command, timestamp FROM shell_history_audit
- `2438` [READ] .query_row("SELECT COUNT(*) FROM shell_history_audit", [], |r| r.get(0))
- `3406` [READ] let mut sql = "SELECT id, substr(command,1,50) as cmd, exit_code, substr(cwd,length(cwd)-20) as cwd,
- `5015` [READ] "SELECT command FROM shell_history WHERE command NOT LIKE 'TIMING:%' AND command NOT LIKE 'SUGGEST:%
- `5086` [READ] "SELECT command, cwd, timestamp, exit_code FROM shell_history
- `6002` [READ] "SELECT command, MAX(timestamp) as ts, COUNT(*) as freq FROM shell_history WHERE command LIKE ?1 AND
- `6210` [READ] .prepare("SELECT command, timestamp FROM shell_history ORDER BY timestamp DESC LIMIT 100")
- `6250` [READ] "SELECT command, timestamp, exit_code FROM shell_history WHERE intent_id = ?1 ORDER BY timestamp ASC
- `6295` [READ] "SELECT COUNT(*) FROM shell_history WHERE intent_id = ?1",
- `6304` [READ] "SELECT COUNT(*) FROM shell_history WHERE intent_id = ?1 AND (exit_code = 0 OR exit_code IS NULL)",
- `6309` [READ] "SELECT command, COUNT(*) as cnt FROM shell_history WHERE intent_id = ?1 GROUP BY command ORDER BY c
- `6345` [UNTYPED] FROM shell_history h
- `6399` [READ] "SELECT command, timestamp FROM shell_history WHERE timestamp >= ?1 ORDER BY timestamp DESC LIMIT 20
- `6434` [READ] "SELECT command, timestamp FROM shell_history WHERE timestamp >= ?1 ORDER BY timestamp ASC",
- `6463` [READ] .prepare("SELECT command, timestamp FROM shell_history ORDER BY timestamp DESC LIMIT 500")
- `6516` [READ] .query_row("SELECT COUNT(*) FROM shell_history", [], |r| r.get(0))
- `6540` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'TIMING:%'",
- `6582` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'grep %' AND timestamp > ?1",
- `6588` [READ] "SELECT COUNT(*) FROM shell_history WHERE (command LIKE 'head %' OR command LIKE 'tail %') AND times
- `6593` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'python3 /tmp/%' AND timestamp > ?1",
- `6598` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'cat % | grep%' AND timestamp > ?1",
- `6644` [READ] "SELECT COUNT(*) FROM shell_history WHERE (command LIKE 'sed %' OR command LIKE '% sed %') AND times
- `6649` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE '%content.replace%' AND timestamp > ?1",
- `8078` [READ] "SELECT command, timestamp FROM shell_history ORDER BY timestamp DESC LIMIT 200",
- `8249` [READ] "SELECT command, timestamp FROM shell_history ORDER BY timestamp DESC LIMIT 500"
- `8601` [READ] "SELECT command, timestamp FROM shell_history ORDER BY timestamp DESC LIMIT 1",
- `8781` [READ] .query_row("SELECT COUNT(*) FROM shell_history", [], |r| r.get(0))
- `8815` [READ] "SELECT command, COUNT(*) as n FROM shell_history GROUP BY command ORDER BY n DESC LIMIT 8",
- `10450` [READ] "SELECT COUNT(*) FROM shell_history WHERE timestamp >= (SELECT COALESCE(MIN(timestamp),0) FROM shell
- `10542` [READ] "SELECT command, COUNT(*) as count FROM shell_history
- `10717` [READ] "SELECT command, COUNT(*) as count FROM shell_history
- `10748` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'fg commit%' AND timestamp > ?1",
- `10761` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'fg commit%' AND timestamp BETWEEN ?1 AND ?2"
- `10793` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'failure_log_%' AND timestamp > ?1",
- `10812` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'deploy %' AND timestamp > ?1",
- `10827` [READ] "SELECT COUNT(*) FROM shell_history WHERE command = 'd' AND timestamp > ?1",
- `10885` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'deploy %' AND timestamp > ?1",
- `10894` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'fg commit%' AND timestamp > ?1",
- `10901` [READ] "SELECT COUNT(*) FROM shell_history WHERE (command LIKE 'cistart%' OR command LIKE 'cicomplete%') AN
- `10908` [READ] "SELECT COUNT(*) FROM shell_history WHERE command = 'd' AND timestamp > ?1",
- `13215` [READ] .prepare("SELECT command FROM shell_history ORDER BY timestamp DESC LIMIT 1000")
- `13285` [READ] .prepare("SELECT timestamp FROM shell_history ORDER BY timestamp DESC LIMIT 2000")
- `16124` [READ] .prepare("SELECT command FROM shell_history ORDER BY timestamp DESC LIMIT 500")
- `16558` [READ] "SELECT COUNT(*) FROM shell_history WHERE timestamp >= ?1",
- `16586` [READ] .query_row("SELECT COUNT(*) FROM shell_history", [], |r| r.get(0))
- `16591` [READ] "SELECT COUNT(*) FROM shell_history WHERE timestamp < ?1",
- `16605` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'SUGGEST:%'",
- `16640` [READ] "SELECT command, COUNT(*) as freq FROM shell_history
- `16689` [READ] SELECT command FROM shell_history

### shell/novashell/src/completion.rs (1)

- `1023` [READ] "SELECT command FROM shell_history              WHERE command LIKE ?1 AND command != ?2 AND length(c

### shell/novashell/src/db.rs (14)

- `125` [UNTYPED] "CREATE TABLE IF NOT EXISTS shell_history (
- `141` [UNTYPED] -- interactive start. Without this the plan is SCAN shell_history + USE TEMP B-TREE
- `152` [UNTYPED] CREATE INDEX IF NOT EXISTS idx_shell_history_timestamp
- `153` [UNTYPED] ON shell_history(timestamp DESC);",
- `159` [UNTYPED] let _ = conn.execute_batch("ALTER TABLE shell_history ADD COLUMN cwd TEXT");
- `160` [UNTYPED] let _ = conn.execute_batch("ALTER TABLE shell_history ADD COLUMN exit_code INTEGER");
- `161` [UNTYPED] let _ = conn.execute_batch("ALTER TABLE shell_history ADD COLUMN duration_ms INTEGER");
- `162` [UNTYPED] let _ = conn.execute_batch("ALTER TABLE shell_history ADD COLUMN intent_id TEXT");
- `204` [UNTYPED] "CREATE TABLE IF NOT EXISTS shell_history_audit (
- `224` [READ] SELECT RAISE(ABORT, 'shell_history_audit is immutable: updates not permitted');
- `229` [READ] SELECT RAISE(ABORT, 'shell_history_audit is immutable: deletes not permitted');
- `253` [READ] .prepare("SELECT command FROM shell_history ORDER BY timestamp DESC LIMIT 10000")
- `421` [READ] "SELECT command FROM shell_history ORDER BY timestamp DESC LIMIT 1",
- `431` [READ] "SELECT command FROM shell_history WHERE command LIKE ?1 ORDER BY timestamp DESC LIMIT 1",

### shell/novashell/src/engine.rs (1)

- `1832` [UNTYPED] FROM shell_history WHERE command LIKE ?1 ORDER BY id DESC LIMIT 20",

### shell/novashell/src/history_tui.rs (2)

- `123` [UNTYPED] FROM shell_history
- `129` [UNTYPED] FROM shell_history

### shell/novashell/src/main.rs (1)

- `2576` [READ] SELECT ?, ?, ?, ?, command FROM shell_history WHERE command NOT LIKE 'TIMING:%' ORDER BY id DESC LIM

### shell/novashell/src/semantic.rs (1)

- `203` [UNTYPED] target: Target::System("shell_history".to_string()),

### tools/db-browse/src/main.rs (2)

- `71` [UNTYPED] "shell_history" => Some('h'),
- `748` [UNTYPED] app.tables.iter().position(|(n, _)| n == "shell_history")

### tools/zero-core/src/state_db.rs (4)

- `17` [UNTYPED] const SENTINEL_TABLE: &str = "shell_history";
- `29` [UNTYPED] #[error("state.db at {} has no ledger schema (no shell_history table)", .0.display())]
- `128` [UNTYPED] .execute_batch("CREATE TABLE shell_history (id INTEGER)")
- `147` [UNTYPED] .execute_batch("CREATE TABLE shell_history (id INTEGER)")

