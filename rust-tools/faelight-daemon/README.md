# faelight-daemon v2.1.0

**Background daemon for Faelight Forest operations** 🌲

## Overview

faelight-daemon is an async background service that handles long-running operations for the Faelight Forest ecosystem. Built with Tokio for efficient async I/O.

## Features

- ✅ Async/await architecture (Tokio runtime)
- ✅ Unix socket communication
- ✅ Clean shutdown handling
- ✅ Zero unwraps - bulletproof error handling
- ✅ Health check endpoint
- ✅ Test client included

## Usage
```bash
# Start daemon (default socket: ~/.local/state/faelight/daemon.sock)
faelight-daemon

# Custom socket path
faelight-daemon --socket /tmp/custom.sock

# Health check
faelight-daemon --health

# Test client
test-client
```

## Architecture

- **main.rs** - CLI and startup (61 lines)
- **daemon.rs** - Core daemon logic (172 lines)
- **test-client.rs** - Testing tool (98 lines)

## Socket Location

Default: `~/.local/state/faelight/daemon.sock`

## Health Check

The daemon includes a built-in health check that verifies:
- Socket availability
- Runtime status
- Connection handling

## Philosophy

Background services should be invisible, reliable, and bulletproof! 🛡️

---

Part of the **Faelight Forest v9.7.0** ecosystem 🌲
