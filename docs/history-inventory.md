# shell_history inventory (INT-191 G1)

GENERATED. Do not edit by hand -- run
`python3 faelight/rust-tools/novashell/generate-history-inventory.py`.

- writers: **4**
- readers: **80**
- untyped (multi-line statements a single line cannot classify): **17**
- matched but NOT consumers: **8**

## Writers

The gate asks whether every history write has a single, well-defined owner.

- `rust-tools/novashell/src/db.rs:338` -- "INSERT INTO shell_history (command, timestamp, cwd, intent_id) VALUES (?1, ?2, ?3, ?4)",
- `rust-tools/novashell/src/db.rs:374` -- "UPDATE shell_history SET exit_code = ?1, duration_ms = ?2 WHERE id = ?3",
- `rust-tools/novashell/src/engine.rs:1943` -- "INSERT INTO shell_history (command, timestamp) VALUES (?1, ?2)",
- `rust-tools/novashell/src/main.rs:2767` -- "INSERT INTO shell_history (command, timestamp) VALUES (?1, strftime('%s','now'))",

## Matched but not consumers

Each was read and judged rather than excluded by a pattern.

- `rust-tools/novashell/src/commands/mod.rs:14541` -- fsh doctor writability probe -- inserts and deletes its own row
- `rust-tools/novashell/src/commands/mod.rs:14546` -- fsh doctor writability probe -- inserts and deletes its own row
- `rust-tools/novashell/src/commands/mod.rs:16464` -- retention pruning, not recording
- `rust-tools/novashell/src/commands/mod.rs:16472` -- retention pruning, not recording
- `rust-tools/novashell/src/db.rs:178` -- trigger definition (shell_history_audit)
- `rust-tools/novashell/src/db.rs:183` -- writes the AUDIT table, not history
- `rust-tools/novashell/src/db.rs:187` -- immutability guard on the audit table
- `rust-tools/novashell/src/db.rs:192` -- immutability guard on the audit table

## Readers, by file

### engine/src/domains/db/mod.rs (1)

- `177` [UNTYPED] "shell_history",

### engine/src/domains/friday/mod.rs (13)

- `739` [READ] "SELECT COUNT(*) FROM shell_history h1
- `742` [READ] SELECT 1 FROM shell_history h2
- `758` [UNTYPED] VALUES ('cicomplete runs', '[\"deploy\", \"workflow\"]', 'deploy tool', 'success', ?1, ?2, ?3, 'shel
- `770` [READ] "SELECT COUNT(*) FROM shell_history h1
- `773` [READ] SELECT 1 FROM shell_history h2
- `789` [UNTYPED] VALUES ('deploy completes', '[\"commit\", \"workflow\"]', 'fg commit', 'success', ?1, ?2, ?3, 'shell
- `801` [READ] "SELECT command, COUNT(*) as cnt FROM shell_history
- `819` [UNTYPED] VALUES ('frequent command', '[\"habit\"]', ?1, 'observed', ?2, 0.8, ?3, 'shell_history')",
- `836` [READ] db.query_row("SELECT COUNT(*) FROM shell_history", [], |r| r
- `932` [READ] "SELECT COUNT(*) FROM shell_history WHERE timestamp > ?1",
- `1219` [READ] "SELECT command FROM shell_history WHERE command LIKE 'python3 /tmp/%'
- `1465` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE ?1 AND timestamp > ?2",
- `1620` [READ] .query_row("SELECT COUNT(*) FROM shell_history", [], |r| r.get(0))

### engine/src/domains/friday/planning.rs (4)

- `447` [READ] "SELECT COUNT(*) FROM shell_history \
- `489` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'cistart%' AND timestamp > ?1",
- `496` [READ] "SELECT COUNT(*) FROM shell_history WHERE timestamp > ?1",
- `821` [READ] "SELECT command FROM shell_history \

### engine/src/domains/friday/reasoning.rs (2)

- `132` [READ] "SELECT COUNT(*) FROM shell_history WHERE timestamp > ?1 AND (command LIKE 'cistart%' OR command LIK
- `138` [READ] "SELECT COUNT(*) FROM shell_history WHERE timestamp > ?1",

### engine/src/domains/friday_arch/mod.rs (3)

- `1113` [READ] "SELECT COUNT(*) FROM shell_history
- `1145` [READ] "SELECT COUNT(*) FROM shell_history
- `1188` [READ] "SELECT COUNT(*) FROM shell_history WHERE command = ?1",

### engine/src/domains/predict/mod.rs (2)

- `1429` [READ] "SELECT command, COUNT(*) as freq FROM shell_history
- `1477` [READ] .query_row("SELECT COUNT(*) FROM shell_history", [], |r| r.get(0))

### rust-tools/db-browse/src/main.rs (2)

- `71` [UNTYPED] "shell_history" => Some('h'),
- `747` [UNTYPED] app.tables.iter().position(|(n, _)| n == "shell_history")

### rust-tools/novashell/src/commands/mod.rs (53)

- `752` [READ] "SELECT command FROM shell_history \
- `796` [READ] "SELECT DISTINCT command FROM shell_history \
- `1930` [READ] "SELECT command FROM shell_history ORDER BY id DESC LIMIT 20"
- `2078` [READ] .prepare("SELECT id, command FROM shell_history ORDER BY id DESC LIMIT ?1")
- `2437` [READ] "SELECT audit_id, command, timestamp FROM shell_history_audit
- `2450` [READ] .query_row("SELECT COUNT(*) FROM shell_history_audit", [], |r| r.get(0))
- `3392` [READ] let mut sql = "SELECT id, substr(command,1,50) as cmd, exit_code, substr(cwd,length(cwd)-20) as cwd,
- `4872` [READ] "SELECT command FROM shell_history WHERE command NOT LIKE 'TIMING:%' AND command NOT LIKE 'SUGGEST:%
- `4943` [READ] "SELECT command, cwd, timestamp, exit_code FROM shell_history
- `5670` [READ] "SELECT command, MAX(timestamp) as ts, COUNT(*) as freq FROM shell_history WHERE command LIKE ?1 AND
- `5878` [READ] .prepare("SELECT command, timestamp FROM shell_history ORDER BY timestamp DESC LIMIT 100")
- `5918` [READ] "SELECT command, timestamp, exit_code FROM shell_history WHERE intent_id = ?1 ORDER BY timestamp ASC
- `5963` [READ] "SELECT COUNT(*) FROM shell_history WHERE intent_id = ?1",
- `5972` [READ] "SELECT COUNT(*) FROM shell_history WHERE intent_id = ?1 AND (exit_code = 0 OR exit_code IS NULL)",
- `5977` [READ] "SELECT command, COUNT(*) as cnt FROM shell_history WHERE intent_id = ?1 GROUP BY command ORDER BY c
- `6013` [UNTYPED] FROM shell_history h
- `6067` [READ] "SELECT command, timestamp FROM shell_history WHERE timestamp >= ?1 ORDER BY timestamp DESC LIMIT 20
- `6102` [READ] "SELECT command, timestamp FROM shell_history WHERE timestamp >= ?1 ORDER BY timestamp ASC",
- `6131` [READ] .prepare("SELECT command, timestamp FROM shell_history ORDER BY timestamp DESC LIMIT 500")
- `6184` [READ] .query_row("SELECT COUNT(*) FROM shell_history", [], |r| r.get(0))
- `6208` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'TIMING:%'",
- `6250` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'grep %' AND timestamp > ?1",
- `6256` [READ] "SELECT COUNT(*) FROM shell_history WHERE (command LIKE 'head %' OR command LIKE 'tail %') AND times
- `6261` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'python3 /tmp/%' AND timestamp > ?1",
- `6266` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'cat % | grep%' AND timestamp > ?1",
- `6312` [READ] "SELECT COUNT(*) FROM shell_history WHERE (command LIKE 'sed %' OR command LIKE '% sed %') AND times
- `6317` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE '%content.replace%' AND timestamp > ?1",
- `7740` [READ] "SELECT command, timestamp FROM shell_history ORDER BY timestamp DESC LIMIT 200",
- `7920` [READ] "SELECT command, timestamp FROM shell_history ORDER BY timestamp DESC LIMIT 500"
- `8378` [READ] "SELECT command, timestamp FROM shell_history ORDER BY timestamp DESC LIMIT 1",
- `8560` [READ] .query_row("SELECT COUNT(*) FROM shell_history", [], |r| r.get(0))
- `8594` [READ] "SELECT command, COUNT(*) as n FROM shell_history GROUP BY command ORDER BY n DESC LIMIT 8",
- `9936` [READ] "SELECT COUNT(*) FROM shell_history WHERE timestamp >= (SELECT COALESCE(MIN(timestamp),0) FROM shell
- `10015` [READ] "SELECT command, COUNT(*) as count FROM shell_history
- `10179` [READ] "SELECT command, COUNT(*) as count FROM shell_history
- `10210` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'fg commit%' AND timestamp > ?1",
- `10223` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'fg commit%' AND timestamp BETWEEN ?1 AND ?2"
- `10255` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'failure_log_%' AND timestamp > ?1",
- `10274` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'deploy %' AND timestamp > ?1",
- `10289` [READ] "SELECT COUNT(*) FROM shell_history WHERE command = 'd' AND timestamp > ?1",
- `10347` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'deploy %' AND timestamp > ?1",
- `10356` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'fg commit%' AND timestamp > ?1",
- `10363` [READ] "SELECT COUNT(*) FROM shell_history WHERE (command LIKE 'cistart%' OR command LIKE 'cicomplete%') AN
- `10370` [READ] "SELECT COUNT(*) FROM shell_history WHERE command = 'd' AND timestamp > ?1",
- `13086` [READ] .prepare("SELECT command FROM shell_history ORDER BY timestamp DESC LIMIT 1000")
- `13156` [READ] .prepare("SELECT timestamp FROM shell_history ORDER BY timestamp DESC LIMIT 2000")
- `15924` [READ] .prepare("SELECT command FROM shell_history ORDER BY timestamp DESC LIMIT 500")
- `16342` [READ] "SELECT COUNT(*) FROM shell_history WHERE timestamp >= ?1",
- `16370` [READ] .query_row("SELECT COUNT(*) FROM shell_history", [], |r| r.get(0))
- `16375` [READ] "SELECT COUNT(*) FROM shell_history WHERE timestamp < ?1",
- `16389` [READ] "SELECT COUNT(*) FROM shell_history WHERE command LIKE 'SUGGEST:%'",
- `16424` [READ] "SELECT command, COUNT(*) as freq FROM shell_history
- `16473` [READ] SELECT command FROM shell_history

### rust-tools/novashell/src/completion.rs (1)

- `1216` [READ] "SELECT command FROM shell_history              WHERE command LIKE ?1 AND command != ?2 AND length(c

### rust-tools/novashell/src/db.rs (11)

- `104` [UNTYPED] "CREATE TABLE IF NOT EXISTS shell_history (
- `124` [UNTYPED] let _ = conn.execute_batch("ALTER TABLE shell_history ADD COLUMN cwd TEXT");
- `125` [UNTYPED] let _ = conn.execute_batch("ALTER TABLE shell_history ADD COLUMN exit_code INTEGER");
- `126` [UNTYPED] let _ = conn.execute_batch("ALTER TABLE shell_history ADD COLUMN duration_ms INTEGER");
- `127` [UNTYPED] let _ = conn.execute_batch("ALTER TABLE shell_history ADD COLUMN intent_id TEXT");
- `169` [UNTYPED] "CREATE TABLE IF NOT EXISTS shell_history_audit (
- `189` [READ] SELECT RAISE(ABORT, 'shell_history_audit is immutable: updates not permitted');
- `194` [READ] SELECT RAISE(ABORT, 'shell_history_audit is immutable: deletes not permitted');
- `218` [READ] .prepare("SELECT command FROM shell_history ORDER BY timestamp DESC LIMIT 10000")
- `428` [READ] "SELECT command FROM shell_history ORDER BY timestamp DESC LIMIT 1",
- `438` [READ] "SELECT command FROM shell_history WHERE command LIKE ?1 ORDER BY timestamp DESC LIMIT 1",

### rust-tools/novashell/src/engine.rs (1)

- `1954` [UNTYPED] FROM shell_history WHERE command LIKE ?1 ORDER BY id DESC LIMIT 20",

### rust-tools/novashell/src/history_tui.rs (2)

- `120` [UNTYPED] FROM shell_history
- `126` [UNTYPED] FROM shell_history

### rust-tools/novashell/src/main.rs (1)

- `2350` [READ] SELECT ?, ?, ?, ?, command FROM shell_history WHERE command NOT LIKE 'TIMING:%' ORDER BY id DESC LIM

### rust-tools/novashell/src/semantic.rs (1)

- `203` [UNTYPED] target: Target::System("shell_history".to_string()),

