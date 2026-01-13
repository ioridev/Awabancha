#![allow(dead_code)]

use anyhow::Result;
use git2::Repository;

/// Tag information
#[derive(Clone, Debug)]
pub struct TagInfo {
    pub name: String,
    pub sha: String,
    pub message: Option<String>,
    pub is_annotated: bool,
}

impl TagInfo {
    pub fn get_all(repo: &Repository) -> Result<Vec<Self>> {
        let mut tags = Vec::new();

        repo.tag_foreach(|oid, name| {
            let name = String::from_utf8_lossy(name)
                .trim_start_matches("refs/tags/")
                .to_string();

            if let Ok(obj) = repo.find_object(oid, None) {
                let (sha, message, is_annotated) = if let Some(tag) = obj.as_tag() {
                    (
                        tag.target_id().to_string(),
                        tag.message().map(|s| s.to_string()),
                        true,
                    )
                } else {
                    (oid.to_string(), None, false)
                };

                tags.push(TagInfo {
                    name,
                    sha,
                    message,
                    is_annotated,
                });
            }

            true
        })?;

        // Sort alphabetically
        tags.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(tags)
    }

    pub fn create_lightweight(repo: &Repository, name: &str, sha: Option<&str>) -> Result<()> {
        let target = if let Some(sha) = sha {
            let oid = git2::Oid::from_str(sha)?;
            repo.find_object(oid, None)?
        } else {
            repo.head()?.peel(git2::ObjectType::Commit)?
        };

        repo.tag_lightweight(name, &target, false)?;
        Ok(())
    }

    pub fn create_annotated(
        repo: &Repository,
        name: &str,
        sha: Option<&str>,
        message: &str,
    ) -> Result<()> {
        let target = if let Some(sha) = sha {
            let oid = git2::Oid::from_str(sha)?;
            repo.find_object(oid, None)?
        } else {
            repo.head()?.peel(git2::ObjectType::Commit)?
        };

        let sig = repo.signature()?;
        repo.tag(name, &target, &sig, message, false)?;
        Ok(())
    }

    pub fn delete(repo: &Repository, name: &str) -> Result<()> {
        repo.tag_delete(name)?;
        Ok(())
    }
}
