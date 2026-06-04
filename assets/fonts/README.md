# assets/fonts/

Build-time font dependency for faelight-lock, faelight-notify, faelight-palette.

These tools use `include_bytes!` to embed the font at compile time.

## TODO (post-1.0.0)
Replace embed with runtime fontconfig lookup so this directory can be removed.
Tracked in INT-036.

## Tools depending on this
- rust-tools/faelight-lock/src/main.rs:337
- rust-tools/faelight-notify/src/render.rs:18
- rust-tools/faelight-palette/src/render.rs:16
