# platform assumption census (INT-227 G1)

GENERATED. Do not edit by hand -- run
`python3 faelight/rust-tools/faelight-shell/generate-platform-census.py`.

⚠️ A COUNT IS NOT A CENSUS. The intent's headline said forty-two places; the
categories below are what reading them actually found.

- **A** (wrong on EVERY system): 0
- **B** (platform capability -- must degrade): 13
- **C** (noise -- not a call): 16
- **D** (already correct): 18
- **OWNER** (the platform module): 2
- **B?** (UNREAD -- needs a human): 0

## B (13)

- `commands/mod.rs:1424` [nix-store] -- real process spawn -- a capability that must degrade
  - `nix_query_lines(&["nix-store", "-q", "--references", "/run/current-system/sw"]);`
- `commands/mod.rs:1424` [/run/current-system] -- real process spawn -- a capability that must degrade
  - `nix_query_lines(&["nix-store", "-q", "--references", "/run/current-system/sw"]);`
- `commands/mod.rs:1470` [nixos-rebuild] -- real process spawn -- a capability that must degrade
  - `let out_raw = std::process::Command::new("nixos-rebuild")`
- `commands/mod.rs:7137` [systemctl] -- real process spawn -- a capability that must degrade
  - `let output = std::process::Command::new("systemctl")`
- `commands/mod.rs:7411` [nix-store] -- real process spawn -- a capability that must degrade
  - `let roots = nix_query_lines(&["nix-store", "--query", "--roots", &path]);`
- `commands/mod.rs:7423` [nix-store] -- real process spawn -- a capability that must degrade
  - `let refs = nix_query_lines(&["nix-store", "--query", "--referrers", &path]);`
- `commands/mod.rs:7499` [nix-store] -- real process spawn -- a capability that must degrade
  - `let dead_out = std::process::Command::new("nix-store")`
- `commands/mod.rs:7583` [nix-store] -- real process spawn -- a capability that must degrade
  - `let roots = nix_query_lines(&["nix-store", "--query", "--roots", p]);`
- `commands/mod.rs:7628` [/nix/store] -- reads the Nix store directly -- a capability that must degrade
  - `if target.starts_with("/nix/store/") && std::path::Path::new(target).exists() {`
- `commands/mod.rs:7633` [/nix/store] -- reads the Nix store directly -- a capability that must degrade
  - `std::fs::read_dir("/nix/store").map_err(|e| format!("  cannot read /nix/store: {}", e))?;`
- `commands/mod.rs:7723` [journalctl] -- real process spawn -- a capability that must degrade
  - `let mut cmd = std::process::Command::new("journalctl");`
- `commands/mod.rs:7760` [journalctl] -- real process spawn -- a capability that must degrade
  - `let output = std::process::Command::new("journalctl")`
- `main.rs:3132` [nixos-rebuild] -- real process spawn -- a capability that must degrade
  - `let nix_gen = std::process::Command::new("nixos-rebuild")`

## D (18)

- `exec.rs:1505` [/home/christian] -- inside a #[test] function -- a literal path is correct in a fixture
  - `assert!(preexec(&ctx, "/home/christian/0-core", &[]).is_some());`
- `exec.rs:1516` [/home/christian] -- inside a #[test] function -- a literal path is correct in a fixture
  - `assert!(preexec(&ctx, "/home/christian/0-core", &[]).is_some());`
- `exec.rs:1524` [/home/christian] -- inside a #[test] function -- a literal path is correct in a fixture
  - `&["cp", "/tmp/thing", "/home/christian/.cargo/bin/core"],`
- `exec.rs:1526` [/home/christian] -- inside a #[test] function -- a literal path is correct in a fixture
  - `assert!(preexec(&ctx, "/home/christian/0-core", &[]).is_some());`
- `exec.rs:1534` [/home/christian] -- inside a #[test] function -- a literal path is correct in a fixture
  - `assert!(preexec(&ctx, "/home/christian/0-core", &[]).is_none());`
- `history/classifier.rs:126` [/home/christian] -- test fixture -- a literal path is correct in an assertion
  - `cwd: "/home/christian/0-core".to_string(),`
- `main.rs:2039` [/run/current-system] -- PATH augmentation -- harmless when the directory is absent
  - `let nix_system = "/run/current-system/sw/bin".to_string();`
- `platform.rs:74` [/nix/store] -- inside a #[test] function -- a literal path is correct in a fixture
  - `id.starts_with("/nix/store/"),`
- `spine/compare.rs:273` [/home/christian] -- inside a #[test] function -- a literal path is correct in a fixture
  - `let legacy = plan_cwd(&["pwd"], Some("/home/christian"));`
- `spine/migrate.rs:106` [/home/christian] -- test fixture -- a literal path is correct in an assertion
  - `cwd: PathBuf::from("/home/christian"),`
- `spine/plan.rs:1285` [/home/christian] -- inside a #[test] function -- a literal path is correct in a fixture
  - `let runner = FakeRunner("/home/christian\n");`
- `spine/plan.rs:1295` [/home/christian] -- inside a #[test] function -- a literal path is correct in a fixture
  - `vec![OsString::from("echo"), OsString::from("/home/christian")]`
- `spine/plan.rs:1337` [/home/christian] -- inside a #[test] function -- a literal path is correct in a fixture
  - `m.insert("HOME", "/home/christian");`
- `spine/plan.rs:1348` [/home/christian] -- inside a #[test] function -- a literal path is correct in a fixture
  - `assert_eq!(a.argv[1], OsString::from("/home/christian"));`
- `triage.rs:224` [/home/christian] -- inside a #[test] function -- a literal path is correct in a fixture
  - `"warning: Git tree '/home/christian/0-core' is dirty"`
- `triage.rs:236` [/home/christian] -- inside a #[test] function -- a literal path is correct in a fixture
  - `"warning: Git tree '/home/christian/0-core' is dirty",`
- `triage.rs:268` [/nix/store] -- inside a #[test] function -- a literal path is correct in a fixture
  - `cold_fix("error: getting status of '/nix/store/x-source/foo.rs': No such file")`
- `triage.rs:294` [/nix/store] -- inside a #[test] function -- a literal path is correct in a fixture
  - `classify("error: getting status of '/nix/store/x/foo.rs': No such file");`

## C (16)

- `commands/mod.rs:1427` [/run/current-system] -- message or predicate text, not a call
  - `"packages: could not read /run/current-system/sw references"`
- `commands/mod.rs:1524` [nix-env] -- message or predicate text, not a call
  - `"\n    sudo nix-env --switch-generation {} -p /nix/var/nix/profiles/system", n`
- `commands/mod.rs:7446` [/run/current-system] -- message or predicate text, not a call
  - `"/run/current-system"])`
- `commands/mod.rs:7505` [/nix/store] -- message or predicate text, not a call
  - `.filter(|l| l.starts_with("/nix/store"))`
- `commands/mod.rs:7510` [nix-store] -- message or predicate text, not a call
  - `format!("  store reclaim: nix-store failed: {}", e).into(),`
- `commands/mod.rs:9866` [systemctl] -- message or predicate text, not a call
  - `"systemctl",`
- `commands/mod.rs:9867` [pacman] -- message or predicate text, not a call
  - `"pacman",`
- `commands/mod.rs:10009` [systemctl] -- message or predicate text, not a call
  - `"systemctl",`
- `commands/mod.rs:10010` [pacman] -- message or predicate text, not a call
  - `"pacman",`
- `commands/mod.rs:11235` [pacman] -- message or predicate text, not a call
  - `"paru" | "pacman" => Some("💡 That isn't a NixOS command — apply changes with deploy (it re`
- `commands/mod.rs:12608` [/run/current-system] -- message or predicate text, not a call
  - `"/run/current-system/sw/bin/faelight-shell".to_string(),`
- `completion.rs:1126` [systemctl] -- message or predicate text, not a call
  - `"systemctl",`
- `completion.rs:1127` [journalctl] -- message or predicate text, not a call
  - `"journalctl",`
- `exec.rs:677` [pacman] -- message or predicate text, not a call
  - `"paru" | "pacman" => Some("💡 That isn't a NixOS command — apply changes with deploy (it re`
- `main.rs:2502` [/run/current-system] -- message or predicate text, not a call
  - `"/run/current-system/sw/bin/faelight-shell".to_string(),`
- `schema.rs:200` [systemctl] -- alias name, not a call
  - `aliases: vec!["svc".to_string(), "systemctl".to_string()],`

## OWNER (2)

- `platform.rs:21` [/run/current-system] -- the platform module itself
  - `std::path::Path::new("/run/current-system/sw/bin").is_dir()`
- `platform.rs:40` [/run/current-system] -- the platform module itself
  - `std::path::PathBuf::from("/run/current-system/sw/bin/faelight-shell")`

