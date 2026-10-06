# claude-statusline

A Claude Code [status line](https://code.claude.com/docs/en/statusline) command that shows the context window fill and a countdown to the prompt cache's expiry:

```
ctx 147k/1M 15% · 4:29
```

- **`ctx`**: `context_window` from the payload Claude Code sends on stdin: tokens in the last response, the model's window, percent used. It is filled right after `/resume`, before any response.
- **clock**: time left on the prompt cache, from `prompt_cache.expires_at` (exact, so it follows whatever TTL is in effect). Shows `cache expired` once it runs out.
- **`~m:ss`**: before a session's first response (a just-resumed session) there is no `prompt_cache` yet, so the clock is estimated from the transcript's last assistant message plus 5 minutes. The `~` marks it as approximate.
- **`ctx --`**: nothing to show yet (fresh session).

Needs Claude Code 2.1.251 or later for `prompt_cache`. Output is plain text: no prefix, no branding.

The clock is computed from the wall clock each time the command runs, so every drawn value is right; a skipped second is only refresh-scheduling jitter.

## Install (Rust)

```bash
cargo build --release
mkdir -p ~/.claude/bin
cp target/release/claude-statusline.exe ~/.claude/bin/   # no .exe on Linux/macOS
```

Add to `~/.claude/settings.json` and restart Claude Code:

```json
"statusLine": {
  "type": "command",
  "command": "~/.claude/bin/claude-statusline.exe",
  "refreshInterval": 1
}
```

`refreshInterval` (seconds, minimum 1) re-runs the command so the clock ticks. If nothing shows, write the full path with forward slashes (`C:/Users/<you>/.claude/bin/claude-statusline.exe`); `~` is expanded by the shell the command runs in.

Copy the exe rather than pointing at `target/release`, so rebuilding is never blocked by a file lock.

## Install (Node)

`statusline.mjs` is the same logic with no build step. Needs Node 18 or later.

```bash
cp statusline.mjs ~/.claude/statusline.mjs
```

```json
"statusLine": {
  "type": "command",
  "command": "node ~/.claude/statusline.mjs",
  "refreshInterval": 1
}
```

## Rust or Node

Same output byte for byte. The command is launched once per refresh, so startup time is what differs. Measured on Windows 11:

| | per launch |
|---|---|
| node | ~133 ms |
| rust | ~14 ms |

Start with Node; switch to Rust if you want the overhead gone. Only the `command` line changes.
