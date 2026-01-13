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
