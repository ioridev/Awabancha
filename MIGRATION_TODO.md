# Awabancha Migration TODO

Migration from VibeWithGraph (Tauri + React) to Awabancha (Rust + gpui)

## Project Overview

- **Reference Implementation**: `/Users/iori/src/VibeWithGraph`
- **Target Implementation**: `/Users/iori/src/Awabancha`
- **UI Framework**: gpui (from Zed editor)
- **Git Library**: git2-rs (libgit2 bindings)

---

## Migration Status

### Phase 1: Project Foundation

- [x] **Cargo.toml setup**
  - Dependencies: gpui, git2, serde, tokio, etc.
  - Build configuration for gpui

- [x] **Main application bootstrap**
  - Application::new() entry point
  - Window creation with proper bounds
  - App activation

- [x] **Project structure**
  ```
  src/
  ├── main.rs           # Entry point
  ├── app.rs            # Main application state
  ├── git/              # Git operations (git2)
  │   ├── mod.rs
  │   ├── repository.rs
  │   ├── commit.rs
  │   ├── branch.rs
  │   ├── status.rs
  │   ├── diff.rs
  │   ├── tag.rs
  │   ├── stash.rs
  │   ├── conflict.rs
  │   └── remote.rs
  ├── views/            # gpui views
  │   ├── mod.rs
  │   ├── welcome.rs
  │   ├── main_layout.rs
  │   ├── left_panel.rs
  │   ├── right_panel.rs
  │   ├── commit_graph.rs
  │   ├── file_list.rs
  │   ├── commit_form.rs
  │   ├── diff_viewer.rs
  │   ├── settings.rs
  │   └── conflict_dialog.rs
  ├── components/       # Reusable UI components
  │   ├── mod.rs
  │   ├── button.rs
  │   ├── input.rs
  │   ├── dropdown.rs
  │   ├── modal.rs
  │   ├── context_menu.rs
  │   └── toast.rs
  ├── state/            # Application state
  │   ├── mod.rs
  │   ├── git_state.rs
  │   ├── settings_state.rs
  │   └── recent_projects.rs
  └── actions.rs        # Keyboard actions
  ```

---

### Phase 2: Welcome Screen

- [x] **Welcome view**
  - Application logo/branding
  - "Open Repository" button (file dialog)
  - Recent projects list (max 10)
  - Last opened timestamp display

- [x] **Recent projects persistence**
  - Save to local storage (JSON file)
  - Load on app start
  - Add project on open
  - Remove project via context menu

- [x] **Drag-and-drop support**
  - Accept dropped directories
  - Validate as git repository
  - Opens repository on valid drop

---

### Phase 3: Git Core Operations

- [x] **Repository management** (git/repository.rs)
  - `open_repository(path)` - Open git repo
  - `get_repository_info()` - HEAD, branches, ahead/behind
  - `close_repository()` - Cleanup

- [x] **File status** (git/status.rs)
  - `get_status()` - List all file changes
  - `stage_file(path)` - Add to index
  - `unstage_file(path)` - Remove from index
  - `stage_all()` - Stage everything
  - `unstage_all()` - Unstage everything
  - `discard_file(path)` - Discard changes
  - `discard_all()` - Discard all changes

- [x] **Commit operations** (git/commit.rs)
  - `create_commit(message)` - New commit
  - `amend_commit(message)` - Modify last commit
  - `get_commit_graph(limit, offset)` - Graph data
  - `get_commit_detail(sha)` - Single commit info
  - `search_commits(query, limit)` - Search by message

- [x] **Diff operations** (git/diff.rs)
  - `get_file_diff(path)` - Working directory diff
  - `get_commit_diff(sha)` - Commit diff

- [x] **Branch operations** (git/branch.rs)
  - `get_branches()` - List branches
  - `checkout_branch(name)` - Switch branch
  - `checkout_commit(sha)` - Detached HEAD
  - `checkout_remote_branch(name)` - Create tracking
  - `create_branch(name)` - New branch
  - `delete_branch(name, force)` - Delete branch
  - `merge_branch(name, mode)` - Merge (auto/ff-only/no-ff/squash)

- [x] **Remote operations** (git/remote.rs)
  - `push(auth)` - Push to remote
  - `pull(auth)` - Fetch + merge
  - `fetch(auth)` - Fetch only
  - `publish_repository(url, auth)` - Setup new remote
  - `set_upstream(auth)` - Set tracking branch
  - Auth handling: HTTPS (token), SSH (agent)

- [x] **Tag operations** (git/tag.rs)
  - `get_tags()` - List tags
  - `create_tag(name, sha, message)` - Create tag
  - `delete_tag(name)` - Delete tag

- [x] **Stash operations** (git/stash.rs)
  - `stash_list()` - List stashes
  - `stash_save(message)` - Save stash
  - `stash_pop(index)` - Apply and remove
  - `stash_apply(index)` - Apply only
  - `stash_drop(index)` - Delete stash

- [x] **Conflict resolution** (git/conflict.rs)
  - `get_merge_conflicts()` - List conflicts
  - `resolve_all_conflicts(strategy)` - Bulk resolve
  - `resolve_conflict_per_file(resolutions)` - Per-file
  - `complete_merge(message)` - Finish merge
  - `abort_merge()` - Cancel merge

- [x] **Advanced operations**
  - `revert_commit(sha, mainline)` - Create undo commit
  - `cherry_pick(sha)` - Apply commit
  - `reset_to_commit(sha, mode)` - Reset HEAD (soft/mixed/hard)

---

### Phase 4: Main Layout

- [x] **3-panel layout** (views/main_layout.rs)
  - Header bar (top)
  - Left panel (changes/commit)
  - Right panel (commit graph/history)

- [x] **Header bar**
  - Current branch display
  - Branch/tag dropdown with search
  - Detached HEAD indicator
  - Ahead/behind count
  - Settings button

---

### Phase 5: Left Panel - Changes & Commit

- [x] **File list** (views/file_list.rs)
  - Virtual scrolling for performance
  - File status icons (added/modified/deleted/renamed/untracked)
  - Color coding by status
  - Click to stage/unstage (UI ready, needs wiring)
  - Double-click to view diff (UI ready, needs wiring)
  - Right-click context menu (discard) - TBD
  - Staged/unstaged sections

- [x] **Commit form** (views/commit_form.rs)
  - Multiline text input for message (UI ready, needs real input)
  - Cmd+Enter to commit (action defined)
  - Amend checkbox
  - Staged file count

- [x] **Remote operations section**
  - Push button (Cmd+Shift+P)
  - Pull button (Cmd+Shift+L)
  - Fetch button
  - Publish repository UI
    - Remote URL input
    - Create new remote / set upstream modes
    - Visibility selector (public/private)

- [x] **Stash management section**
  - Stash list with collapse/expand
  - Apply/pop/drop actions
  - Save stash with message

---

### Phase 6: Right Panel - Commit Graph

- [x] **Commit graph visualization** (views/commit_graph.rs)
  - ASCII-art style branch visualization
  - Node colors per branch
  - Parent-child relationship lines
  - Merge commit display
  - HEAD indicator (blue circle)
  - Branch/remote/tag labels on commits

- [x] **Commit list**
  - Virtual scrolling
  - Author, timestamp, message
  - Relative date display
  - Click to select
  - Double-click to checkout (needs wiring)

- [x] **Search functionality**
  - Search commits by message
  - Debounced search (300ms)
  - Clear search button

- [x] **Context menu** (components/context_menu.rs)
  - Checkout commit
  - Revert commit
  - Cherry-pick
  - Merge (with mode selector)
  - Create branch from commit
  - Create tag from commit
  - Reset to commit (soft/mixed/hard)

- [x] **Infinite scroll / pagination**
  - Load commits in batches (100)
  - "Load More" button

---

### Phase 7: Diff Viewer

- [x] **Diff viewer modal** (views/diff_viewer.rs)
  - Modal overlay
  - Syntax highlighting (add/delete/context)
  - Old/new line numbers
  - Addition/deletion counters
  - Virtual scrolling for large diffs
  - Escape to close

---

### Phase 8: Settings

- [x] **Settings modal** (views/settings.rs)
  - Git authentication section
    - Auth mode (HTTPS/SSH)
    - Username input
    - Token input (password field)
    - Paste from clipboard
  - Merge options
    - Default merge mode selector
  - UI settings
    - Language (en, ja, zh-Hans, zh-Hant)
    - Theme (dark/light) - dark only initially
  - About section
    - Version info
    - Links

- [x] **Settings persistence**
  - Save to JSON file
  - Load on app start

---

### Phase 9: Conflict Resolution

- [x] **Conflict resolution dialog** (views/conflict_dialog.rs)
  - Mode selector: bulk vs per-file
  - Conflicted files list
  - Deleted-by-us/them indicators
  - Strategy buttons (ours/theirs)
  - Abort merge option
  - Complete merge button

---

### Phase 10: Keyboard Shortcuts

- [x] **Actions** (actions.rs)
  - `StageAll` - Cmd+S
  - `CreateCommit` - Cmd+Enter
  - `Push` - Cmd+Shift+P
  - `Pull` - Cmd+Shift+L
  - `Refresh` - Cmd+R
  - `CloseModal` - Escape
  - `OpenRepository` - Cmd+O
  - `OpenSettings` - Cmd+,

---

### Phase 11: UI Components

- [x] **Button** (components/button.rs)
  - Primary/secondary variants
  - Disabled state
  - Loading state
  - Icon support

- [x] **Input** (components/input.rs)
  - Text input
  - Password input (masked)
  - Multiline textarea
  - Placeholder text
  - Focus management (needs real text editor)

- [x] **Dropdown** (components/dropdown.rs)
  - Options list
  - Search/filter (needs implementation)
  - Selected state

- [x] **Modal** (components/modal.rs)
  - Overlay backdrop
  - Close on escape
  - Close on click outside

- [x] **Toast notifications** (components/toast.rs)
  - Success/error/warning/info types
  - Auto-dismiss
  - Manual dismiss

---

### Phase 12: File Watching

- [x] **Repository watcher**
  - Monitor .git directory
  - Debounced updates (500ms)
  - Filter temporary files
  - Trigger refresh on changes

---

### Phase 13: Internationalization (Future)

- [ ] **i18n support**
  - English
  - Japanese
  - Simplified Chinese
  - Traditional Chinese

---

## Implementation Progress Tracker

| Feature | Status | Notes |
|---------|--------|-------|
| Project setup | 🟢 Done | Cargo.toml, directory structure |
| App bootstrap | 🟢 Done | main.rs, app.rs with gpui |
| Welcome screen | 🟢 Done | Recent projects, open button, drag-drop |
| Git repository ops | 🟢 Done | git2 integration |
| File status ops | 🟢 Done | stage/unstage/discard |
| Commit ops | 🟢 Done | create/amend with message loading |
| Branch ops | 🟢 Done | checkout/create/delete |
| Remote ops | 🟢 Done | push/pull/fetch |
| Advanced ops | 🟢 Done | revert/cherry-pick/reset |
| Main layout | 🟢 Done | 3-panel with header |
| Left panel | 🟢 Done | File list + commit form + stash |
| Right panel | 🟢 Done | Commit graph + search |
| Commit graph | 🟢 Done | Visual graph with branches |
| Context menu | 🟢 Done | Checkout, branch/tag inline forms, cherry-pick, revert, reset |
| Diff viewer | 🟢 Done | Line-by-line diff, modal on double-click |
| Settings | 🟢 Done | Auth mode, merge options, keyboard shortcuts |
| Conflict resolution | 🟢 Done | git2 conflict APIs, modal dialog |
| Keyboard shortcuts | 🟢 Done | Actions registered |
| File watching | 🟢 Done | Auto-refresh on changes |
| Window focus refresh | 🟢 Done | Auto-refresh on window activation |
| Drag & Drop | 🟢 Done | Open repos via file drop |
| Toast notifications | 🟢 Done | Success/error feedback with auto-dismiss |
| Commit search | 🟢 Done | Search by message, author, SHA |
| i18n | 🔴 Not Started | Future |

**Legend:** 🟢 Done | 🟡 Partial | 🔴 Not Started

---

## Dependencies

```toml
[dependencies]
gpui = { path = "../zed/crates/gpui" }
git2 = "0.18"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
chrono = "0.4"
dirs = "5.0"
notify = "6.0"
thiserror = "1.0"
anyhow = "1.0"
```

---

## Architecture Notes

### gpui Patterns Used

1. **Entity-based state**: All git state owned by App context
2. **Observer pattern**: Views react to state changes via `cx.observe()`
3. **Actions for keybindings**: Define actions, bind keys, handle in views
4. **Div-based layout**: Flexbox/Grid layout with Tailwind-like API
5. **Virtual lists**: `uniform_list` for large commit/file lists

### Data Flow

```
User Action → Event Handler → Git Operation → State Update → cx.notify() → View Re-render
```

### Thread Model

- Main thread: UI rendering (gpui)
- Background thread: Git operations (via cx.spawn)
- Watcher thread: File system monitoring
