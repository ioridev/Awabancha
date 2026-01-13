use gpui::*;

// Git operations
actions!(
    awabancha,
    [
        StageAll,
        UnstageAll,
        DiscardAll,
        CreateCommit,
        AmendCommit,
        Push,
        Pull,
        Fetch,
        Refresh,
    ]
);

// Navigation
actions!(
    awabancha,
    [
        OpenRepository,
        CloseRepository,
        OpenSettings,
        CloseModal,
        Cancel,
    ]
);

// Branch operations
actions!(
    awabancha,
    [
        CreateBranch,
        DeleteBranch,
        CheckoutBranch,
        MergeBranch,
    ]
);

// Tag operations
actions!(awabancha, [CreateTag, DeleteTag,]);

// Stash operations
actions!(awabancha, [StashSave, StashPop, StashApply, StashDrop,]);

// Text input actions
actions!(
    text_input,
    [
        Backspace,
        Delete,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        ShowCharacterPalette,
        Paste,
        Cut,
        Copy,
        Enter,
    ]
);

pub fn register_actions(cx: &mut App) {
    // Register keybindings
    cx.bind_keys([
        // Git operations
        KeyBinding::new("cmd-s", StageAll, None),
        KeyBinding::new("cmd-enter", CreateCommit, None),
        KeyBinding::new("cmd-shift-p", Push, None),
        KeyBinding::new("cmd-shift-l", Pull, None),
        KeyBinding::new("cmd-r", Refresh, None),
        // Navigation
        KeyBinding::new("cmd-o", OpenRepository, None),
        KeyBinding::new("cmd-,", OpenSettings, None),
        KeyBinding::new("escape", Cancel, None),
        // Text input
        KeyBinding::new("backspace", Backspace, Some("TextInput")),
        KeyBinding::new("delete", Delete, Some("TextInput")),
        KeyBinding::new("left", Left, Some("TextInput")),
        KeyBinding::new("right", Right, Some("TextInput")),
        KeyBinding::new("shift-left", SelectLeft, Some("TextInput")),
        KeyBinding::new("shift-right", SelectRight, Some("TextInput")),
        KeyBinding::new("cmd-a", SelectAll, Some("TextInput")),
        KeyBinding::new("cmd-v", Paste, Some("TextInput")),
        KeyBinding::new("cmd-c", Copy, Some("TextInput")),
        KeyBinding::new("cmd-x", Cut, Some("TextInput")),
        KeyBinding::new("home", Home, Some("TextInput")),
        KeyBinding::new("end", End, Some("TextInput")),
        KeyBinding::new("ctrl-cmd-space", ShowCharacterPalette, Some("TextInput")),
        KeyBinding::new("enter", Enter, Some("TextInput")),
    ]);
}
