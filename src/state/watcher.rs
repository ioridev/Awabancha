#![allow(dead_code)]

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::time::Duration;

/// File system watcher for repository changes
pub struct RepositoryWatcher {
    watcher: Option<RecommendedWatcher>,
    stop_flag: Arc<AtomicBool>,
    receiver: Option<mpsc::Receiver<()>>,
    watched_path: Option<PathBuf>,
}

impl RepositoryWatcher {
    pub fn new() -> Self {
        Self {
            watcher: None,
            stop_flag: Arc::new(AtomicBool::new(false)),
            receiver: None,
            watched_path: None,
        }
    }

    /// Start watching a repository path
    pub fn watch(&mut self, path: PathBuf) -> anyhow::Result<()> {
        // Stop any existing watcher
        self.stop();

        // Create channels for event passing
        let (event_tx, event_rx) = mpsc::channel::<()>();
        let (debounced_tx, debounced_rx) = mpsc::channel::<()>();

        self.stop_flag.store(false, Ordering::SeqCst);
        let stop_flag = self.stop_flag.clone();

        // Create the watcher
        let watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    if Self::should_notify(&event) {
                        let _ = event_tx.send(());
                    }
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(1)),
        )?;

        // Spawn debounce thread
        std::thread::spawn(move || {
            let debounce_duration = Duration::from_millis(500);
            let mut last_event = None;

            loop {
                if stop_flag.load(Ordering::SeqCst) {
                    break;
                }

                match event_rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(()) => {
                        last_event = Some(std::time::Instant::now());
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if let Some(last) = last_event {
                            if last.elapsed() >= debounce_duration {
                                let _ = debounced_tx.send(());
                                last_event = None;
                            }
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        break;
                    }
                }
            }
        });

        let mut watcher = watcher;

        // Watch the .git directory
        let git_path = path.join(".git");
        if git_path.exists() {
            watcher.watch(&git_path, RecursiveMode::Recursive)?;
        }

        // Watch the working directory for file changes (non-recursive to avoid noise)
        watcher.watch(&path, RecursiveMode::NonRecursive)?;

        self.watcher = Some(watcher);
        self.receiver = Some(debounced_rx);
        self.watched_path = Some(path);

        Ok(())
    }

    /// Check if there's a pending refresh notification
    /// This should be called periodically (e.g., in a background task)
    pub fn poll(&self) -> bool {
        if let Some(ref rx) = self.receiver {
            matches!(rx.try_recv(), Ok(()))
        } else {
            false
        }
    }

    /// Stop watching
    pub fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        self.watcher = None;
        self.receiver = None;
        self.watched_path = None;
    }

    /// Check if currently watching
    pub fn is_watching(&self) -> bool {
        self.watcher.is_some()
    }

    /// Check if an event should trigger a notification
    fn should_notify(event: &notify::Event) -> bool {
        use notify::EventKind;

        match &event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                for path in &event.paths {
                    // Skip if it's in the .git directory but not a relevant file
                    let path_str = path.to_string_lossy();

                    // Skip pack files, logs, etc. that change frequently
                    if path_str.contains(".git/objects/pack")
                        || path_str.contains(".git/logs")
                        || path_str.contains(".git/COMMIT_EDITMSG")
                        || path_str.contains(".git/FETCH_HEAD")
                    {
                        continue;
                    }

                    // Check for important .git changes
                    if path_str.contains(".git/") {
                        // These indicate actual git state changes
                        if path_str.contains(".git/index")
                            || path_str.contains(".git/HEAD")
                            || path_str.contains(".git/refs/")
                            || path_str.contains(".git/MERGE_HEAD")
                            || path_str.contains(".git/CHERRY_PICK_HEAD")
                        {
                            return true;
                        }
                        continue;
                    }

                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        // Skip swap files, backup files, IDE files
                        if filename.ends_with('~')
                            || filename.ends_with(".swp")
                            || filename.ends_with(".swx")
                            || filename.starts_with(".#")
                            || filename == "4913"
                        {
                            continue;
                        }

                        // Skip build directories
                        if path_str.contains("node_modules")
                            || path_str.contains("target/debug")
                            || path_str.contains("target/release")
                            || path_str.contains(".next")
                            || path_str.contains(".nuxt")
                        {
                            continue;
                        }

                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }
}

impl Default for RepositoryWatcher {
    fn default() -> Self {
        Self::new()
    }
}
