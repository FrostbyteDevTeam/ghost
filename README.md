# Ghost

Windows desktop automation framework. Like Playwright, but for native apps.

Any application. Any input. Any agent.

## What is Ghost?

Ghost gives AI agents and developers programmatic control over any Windows application — native Win32, Electron, WPF, or otherwise. It uses the Windows UI Automation API for element discovery, Win32 SendInput for keyboard/mouse injection, and DXGI for screen capture.

## Quick Start

```toml
# Cargo.toml
[dependencies]
ghost-session = { git = "https://github.com/FrostbyteDevTeam/ghost" }
```

```rust
use ghost_session::{GhostSession, By, session::Region};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = GhostSession::new()?;

    // Launch and find
    session.launch("notepad.exe").await?;
    let edit = session.find(By::role("edit")).await?;
    edit.type_text("hello world")?;

    // Screenshot
    let png = session.screenshot(Region::full()).await?;
    std::fs::write("screen.png", png)?;

    Ok(())
}
```

## Emergency Stop

Press **Ctrl+Alt+G** at any time to immediately halt all automation.
- All queued actions are cancelled
- Any held modifier keys (Shift, Ctrl, Alt) are released immediately
- No stuck keys, no stuck modifier states

## MCP Server (for AI agents)

Build and add to Claude Code as an MCP server:

```bash
cargo build -p ghost-mcp --release
```

Add to Claude Code settings:
```json
{
  "mcpServers": {
    "ghost": {
      "command": "path/to/ghost-mcp.exe"
    }
  }
}
```

Available tools: `ghost_find`, `ghost_click`, `ghost_type`, `ghost_click_at`, `ghost_screenshot`, `ghost_launch`, `ghost_stop`

## Element Locators

```rust
// By accessible name (case-insensitive substring)
session.find(By::name("Save")).await?

// By control type role
session.find(By::role("edit")).await?    // text input
session.find(By::role("button")).await?  // button
session.find(By::role("checkbox")).await?
session.find(By::role("list")).await?
```

## Architecture

```
ghost-session  ← developer/agent API (safe Rust)
     │
ghost-core     ← Win32 FFI: UIA, SendInput, DXGI (unsafe Rust)
     │
Windows OS     ← UIA COM, user32.dll, DXGI
```

## Requirements

- Windows 10 or later
- Rust stable

## License

MIT - Copyright 2026 Frostbyte Digital
