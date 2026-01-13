# Awabancha

A lightweight Git GUI client built with [gpui](https://github.com/zed-industries/zed/tree/main/crates/gpui).  

<img width="1224" height="1042" alt="image" src="https://github.com/user-attachments/assets/48be0280-4c53-4084-a940-1db0f5872e49" />


## Features

- **Fast & Native**: Built with Rust and gpui for high performance
- **Git Operations**: Stage, commit, push, pull, fetch, stash, merge, revert, cherry-pick, reset
- **Commit Graph**: Visual branch/merge history with ASCII-style graph
- **Diff Viewer**: Line-by-line diff with syntax highlighting
- **Branch Management**: Create, checkout, delete branches and tags
- **Conflict Resolution**: Bulk or per-file merge conflict resolution
- **Search**: Find commits by message, author, or SHA
- **i18n**: English, Japanese, Simplified Chinese, Traditional Chinese

## Requirements

- macOS (gpui currently supports macOS only)
- Rust 1.75+

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Cmd+O | Open Repository |
| Cmd+S | Stage All |
| Cmd+Enter | Commit |
| Cmd+Shift+P | Push |
| Cmd+Shift+L | Pull |
| Cmd+R | Refresh |
| Cmd+, | Settings |
| Escape | Close Modal |

## License

MIT
