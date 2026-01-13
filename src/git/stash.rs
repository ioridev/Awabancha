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
