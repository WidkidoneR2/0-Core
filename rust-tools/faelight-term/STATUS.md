# faelight-term Status

## Working ✅
- PTY spawning and shell integration
- Font rendering with fontdue
- ANSI escape code parsing
- Wayland protocol (verified with WAYLAND_DEBUG)

## Bug 🐛
Window doesn't appear despite perfect protocol.
Verified: ack_configure + buffer attach + commit all correct.
Next: Try minimal C wayland-client to isolate Rust/smithay issue.

## Post-Linus TODO
- [ ] Test with weston compositor
- [ ] Compare with working terminal (alacritty source)
- [ ] Try raw wayland-client-rs
