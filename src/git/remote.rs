#![allow(dead_code)]

use anyhow::Result;
use git2::Repository;

/// Remote information
#[derive(Clone, Debug)]
pub struct RemoteInfo {
    pub name: String,
    pub url: String,
    pub push_url: Option<String>,
}

impl RemoteInfo {
    pub fn get_all(repo: &Repository) -> Result<Vec<Self>> {
        let mut remotes = Vec::new();

        let remote_names = repo.remotes()?;
        for name in remote_names.iter().flatten() {
            if let Ok(remote) = repo.find_remote(name) {
                remotes.push(RemoteInfo {
                    name: name.to_string(),
                    url: remote.url().unwrap_or("").to_string(),
                    push_url: remote.pushurl().map(|s| s.to_string()),
                });
            }
        }

        Ok(remotes)
    }

    pub fn add(repo: &Repository, name: &str, url: &str) -> Result<()> {
        repo.remote(name, url)?;
        Ok(())
    }

    pub fn remove(repo: &Repository, name: &str) -> Result<()> {
        repo.remote_delete(name)?;
        Ok(())
    }

    pub fn set_url(repo: &Repository, name: &str, url: &str) -> Result<()> {
        repo.remote_set_url(name, url)?;
        Ok(())
    }
}

/// Auth credentials for remote operations
pub struct RemoteAuth {
    pub username: String,
    pub password: String,
}

impl RemoteAuth {
    pub fn create_callbacks(&self) -> git2::RemoteCallbacks<'_> {
        let mut callbacks = git2::RemoteCallbacks::new();
        let username = self.username.clone();
        let password = self.password.clone();

        callbacks.credentials(move |_url, _username_from_url, allowed_types| {
            if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
                git2::Cred::userpass_plaintext(&username, &password)
            } else if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                // Try SSH agent
                git2::Cred::ssh_key_from_agent(&username)
            } else {
                Err(git2::Error::from_str("No suitable authentication method"))
            }
        });

        callbacks
    }
}

pub fn push_to_remote(
    repo: &Repository,
    remote_name: &str,
    branch_name: &str,
    auth: Option<&RemoteAuth>,
) -> Result<()> {
    let mut remote = repo.find_remote(remote_name)?;

    let mut push_opts = git2::PushOptions::new();
    if let Some(auth) = auth {
        push_opts.remote_callbacks(auth.create_callbacks());
    }

    let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
    remote.push(&[&refspec], Some(&mut push_opts))?;

    Ok(())
}

pub fn fetch_from_remote(
    repo: &Repository,
    remote_name: &str,
    auth: Option<&RemoteAuth>,
) -> Result<()> {
    let mut remote = repo.find_remote(remote_name)?;

    let mut fetch_opts = git2::FetchOptions::new();
    if let Some(auth) = auth {
        fetch_opts.remote_callbacks(auth.create_callbacks());
    }

    remote.fetch::<&str>(&[], Some(&mut fetch_opts), None)?;

    Ok(())
}

pub fn pull_from_remote(
    repo: &Repository,
    remote_name: &str,
    branch_name: &str,
    auth: Option<&RemoteAuth>,
) -> Result<()> {
    // First fetch
    fetch_from_remote(repo, remote_name, auth)?;

    // Then merge
    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;

    let (analysis, _) = repo.merge_analysis(&[&fetch_commit])?;

    if analysis.is_up_to_date() {
        return Ok(());
    }

    if analysis.is_fast_forward() {
        let mut reference = repo.find_reference(&format!("refs/heads/{}", branch_name))?;
        reference.set_target(fetch_commit.id(), "Fast-forward pull")?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;
    } else if analysis.is_normal() {
        repo.merge(&[&fetch_commit], None, None)?;
    }

    Ok(())
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

    #[test]
    fn test_remote_info_creation() {
        let info = RemoteInfo {
            name: "origin".to_string(),
            url: "https://github.com/user/repo.git".to_string(),
            push_url: None,
        };

        assert_eq!(info.name, "origin");
        assert_eq!(info.url, "https://github.com/user/repo.git");
        assert!(info.push_url.is_none());
    }

    #[test]
    fn test_remote_info_with_push_url() {
        let info = RemoteInfo {
            name: "origin".to_string(),
            url: "git@github.com:user/repo.git".to_string(),
            push_url: Some("git@github.com:user/repo.git".to_string()),
        };

        assert_eq!(info.name, "origin");
        assert!(info.push_url.is_some());
    }

    #[test]
    fn test_remote_info_clone() {
        let info = RemoteInfo {
            name: "upstream".to_string(),
            url: "https://github.com/original/repo.git".to_string(),
            push_url: Some("https://github.com/fork/repo.git".to_string()),
        };

        let cloned = info.clone();
        assert_eq!(info.name, cloned.name);
        assert_eq!(info.url, cloned.url);
        assert_eq!(info.push_url, cloned.push_url);
    }

    #[test]
    fn test_get_all_remotes_empty() {
        let (_temp_dir, repo) = create_test_repo();

        let remotes = RemoteInfo::get_all(&repo).unwrap();
        assert!(remotes.is_empty());
    }

    #[test]
    fn test_add_remote() {
        let (_temp_dir, repo) = create_test_repo();

        let result = RemoteInfo::add(&repo, "origin", "https://github.com/test/repo.git");
        assert!(result.is_ok());

        let remotes = RemoteInfo::get_all(&repo).unwrap();
        assert_eq!(remotes.len(), 1);
        assert_eq!(remotes[0].name, "origin");
        assert_eq!(remotes[0].url, "https://github.com/test/repo.git");
    }

    #[test]
    fn test_remove_remote() {
        let (_temp_dir, repo) = create_test_repo();

        RemoteInfo::add(&repo, "origin", "https://github.com/test/repo.git").unwrap();
        assert_eq!(RemoteInfo::get_all(&repo).unwrap().len(), 1);

        let result = RemoteInfo::remove(&repo, "origin");
        assert!(result.is_ok());
        assert_eq!(RemoteInfo::get_all(&repo).unwrap().len(), 0);
    }

    #[test]
    fn test_set_remote_url() {
        let (_temp_dir, repo) = create_test_repo();

        RemoteInfo::add(&repo, "origin", "https://github.com/test/repo.git").unwrap();

        let result = RemoteInfo::set_url(&repo, "origin", "https://github.com/new/repo.git");
        assert!(result.is_ok());

        let remotes = RemoteInfo::get_all(&repo).unwrap();
        assert_eq!(remotes[0].url, "https://github.com/new/repo.git");
    }

    #[test]
    fn test_multiple_remotes() {
        let (_temp_dir, repo) = create_test_repo();

        RemoteInfo::add(&repo, "origin", "https://github.com/user/repo.git").unwrap();
        RemoteInfo::add(&repo, "upstream", "https://github.com/original/repo.git").unwrap();

        let remotes = RemoteInfo::get_all(&repo).unwrap();
        assert_eq!(remotes.len(), 2);

        let names: Vec<&str> = remotes.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"origin"));
        assert!(names.contains(&"upstream"));
    }

    #[test]
    fn test_remote_auth_creation() {
        let auth = RemoteAuth {
            username: "testuser".to_string(),
            password: "testpassword".to_string(),
        };

        assert_eq!(auth.username, "testuser");
        assert_eq!(auth.password, "testpassword");
    }

    #[test]
    fn test_remote_auth_create_callbacks() {
        let auth = RemoteAuth {
            username: "user".to_string(),
            password: "pass".to_string(),
        };

        // Just verify it doesn't panic
        let _callbacks = auth.create_callbacks();
    }
}
