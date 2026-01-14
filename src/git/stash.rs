#![allow(dead_code)]

use anyhow::Result;
use git2::Repository;

/// Stash entry
#[derive(Clone, Debug)]
pub struct StashEntry {
    pub index: usize,
    pub message: String,
    pub oid: String,
}

impl StashEntry {
    pub fn get_all(repo: &mut Repository) -> Result<Vec<Self>> {
        let mut stashes = Vec::new();

        repo.stash_foreach(|index, message, oid| {
            stashes.push(StashEntry {
                index,
                message: message.to_string(),
                oid: oid.to_string(),
            });
            true
        })?;

        Ok(stashes)
    }

    pub fn save(repo: &mut Repository, message: Option<&str>) -> Result<()> {
        let sig = repo.signature()?;
        let msg = message.unwrap_or("WIP");
        repo.stash_save(&sig, msg, None)?;
        Ok(())
    }

    pub fn pop(repo: &mut Repository, index: usize) -> Result<()> {
        repo.stash_pop(index, None)?;
        Ok(())
    }

    pub fn apply(repo: &mut Repository, index: usize) -> Result<()> {
        repo.stash_apply(index, None)?;
        Ok(())
    }

    pub fn drop(repo: &mut Repository, index: usize) -> Result<()> {
        repo.stash_drop(index)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, Repository) {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();

        (temp_dir, repo)
    }

    fn create_initial_commit(temp_dir: &TempDir, repo: &Repository) {
        // Create a file and commit it
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "initial content").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();

        let sig = repo.signature().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();

        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap();
    }

    #[test]
    fn test_stash_entry_creation() {
        let entry = StashEntry {
            index: 0,
            message: "WIP: feature work".to_string(),
            oid: "abc123".to_string(),
        };

        assert_eq!(entry.index, 0);
        assert_eq!(entry.message, "WIP: feature work");
        assert_eq!(entry.oid, "abc123");
    }

    #[test]
    fn test_stash_entry_clone() {
        let entry = StashEntry {
            index: 1,
            message: "test stash".to_string(),
            oid: "def456".to_string(),
        };

        let cloned = entry.clone();
        assert_eq!(entry.index, cloned.index);
        assert_eq!(entry.message, cloned.message);
        assert_eq!(entry.oid, cloned.oid);
    }

    #[test]
    fn test_get_all_stashes_empty() {
        let (_temp_dir, mut repo) = create_test_repo();

        let stashes = StashEntry::get_all(&mut repo).unwrap();
        assert!(stashes.is_empty());
    }

    #[test]
    fn test_stash_save_and_get() {
        let (temp_dir, mut repo) = create_test_repo();
        create_initial_commit(&temp_dir, &repo);

        // Modify the file to create something to stash
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "modified content").unwrap();

        // Stage the change
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();

        // Save stash
        let result = StashEntry::save(&mut repo, Some("Test stash"));
        assert!(result.is_ok());

        // Get all stashes
        let stashes = StashEntry::get_all(&mut repo).unwrap();
        assert_eq!(stashes.len(), 1);
        assert!(stashes[0].message.contains("Test stash"));
    }

    #[test]
    fn test_stash_drop() {
        let (temp_dir, mut repo) = create_test_repo();
        create_initial_commit(&temp_dir, &repo);

        // Modify and stash
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "modified content").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();

        StashEntry::save(&mut repo, Some("Test stash")).unwrap();
        assert_eq!(StashEntry::get_all(&mut repo).unwrap().len(), 1);

        // Drop the stash
        let result = StashEntry::drop(&mut repo, 0);
        assert!(result.is_ok());
        assert_eq!(StashEntry::get_all(&mut repo).unwrap().len(), 0);
    }
}
