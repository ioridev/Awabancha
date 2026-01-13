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
    ]);
}
