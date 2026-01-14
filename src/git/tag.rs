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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, Repository) {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();

        (temp_dir, repo)
    }

    fn create_initial_commit(repo: &Repository) -> git2::Oid {
        let sig = repo.signature().unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();

        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap()
    }

    #[test]
    fn test_tag_info_creation() {
        let tag = TagInfo {
            name: "v1.0.0".to_string(),
            sha: "abc123def456".to_string(),
            message: Some("Release version 1.0.0".to_string()),
            is_annotated: true,
        };

        assert_eq!(tag.name, "v1.0.0");
        assert_eq!(tag.sha, "abc123def456");
        assert_eq!(tag.message, Some("Release version 1.0.0".to_string()));
        assert!(tag.is_annotated);
    }

    #[test]
    fn test_tag_info_lightweight() {
        let tag = TagInfo {
            name: "v0.1.0".to_string(),
            sha: "xyz789".to_string(),
            message: None,
            is_annotated: false,
        };

        assert_eq!(tag.name, "v0.1.0");
        assert!(tag.message.is_none());
        assert!(!tag.is_annotated);
    }

    #[test]
    fn test_tag_info_clone() {
        let tag = TagInfo {
            name: "v1.0.0".to_string(),
            sha: "abc123".to_string(),
            message: Some("Test".to_string()),
            is_annotated: true,
        };

        let cloned = tag.clone();
        assert_eq!(tag.name, cloned.name);
        assert_eq!(tag.sha, cloned.sha);
        assert_eq!(tag.message, cloned.message);
        assert_eq!(tag.is_annotated, cloned.is_annotated);
    }

    #[test]
    fn test_create_lightweight_tag() {
        let (_temp_dir, repo) = create_test_repo();
        create_initial_commit(&repo);

        let result = TagInfo::create_lightweight(&repo, "v0.1.0", None);
        assert!(result.is_ok());

        let tags = TagInfo::get_all(&repo).unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "v0.1.0");
        assert!(!tags[0].is_annotated);
    }

    #[test]
    fn test_create_annotated_tag() {
        let (_temp_dir, repo) = create_test_repo();
        create_initial_commit(&repo);

        let result = TagInfo::create_annotated(&repo, "v1.0.0", None, "Release 1.0.0");
        assert!(result.is_ok());

        let tags = TagInfo::get_all(&repo).unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "v1.0.0");
        assert!(tags[0].is_annotated);
    }

    #[test]
    fn test_delete_tag() {
        let (_temp_dir, repo) = create_test_repo();
        create_initial_commit(&repo);

        TagInfo::create_lightweight(&repo, "v0.1.0", None).unwrap();
        assert_eq!(TagInfo::get_all(&repo).unwrap().len(), 1);

        let result = TagInfo::delete(&repo, "v0.1.0");
        assert!(result.is_ok());
        assert_eq!(TagInfo::get_all(&repo).unwrap().len(), 0);
    }

    #[test]
    fn test_get_all_tags_empty() {
        let (_temp_dir, repo) = create_test_repo();
        create_initial_commit(&repo);

        let tags = TagInfo::get_all(&repo).unwrap();
        assert!(tags.is_empty());
    }

    #[test]
    fn test_get_all_tags_sorted() {
        let (_temp_dir, repo) = create_test_repo();
        create_initial_commit(&repo);

        TagInfo::create_lightweight(&repo, "v2.0.0", None).unwrap();
        TagInfo::create_lightweight(&repo, "v1.0.0", None).unwrap();
        TagInfo::create_lightweight(&repo, "v1.5.0", None).unwrap();

        let tags = TagInfo::get_all(&repo).unwrap();
        assert_eq!(tags.len(), 3);
        assert_eq!(tags[0].name, "v1.0.0");
        assert_eq!(tags[1].name, "v1.5.0");
        assert_eq!(tags[2].name, "v2.0.0");
    }
}
