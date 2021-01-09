pub mod git;
pub mod playlist;

use crate::git::Git;
use crate::playlist::Playlist;
use anyhow::{Context, Result};
use chrono::Utc;
use configparser::ini::Ini;
use git2::Signature;

fn main() -> Result<()> {
    let mut config = Ini::new();
    config
        .load("example.ini")
        .expect("failed to load configuration file");
    let repo_path = config
        .get("general", "git-repo-path")
        .context("repository not specified")?;
    let signature = Signature::now(
        &(config
            .get("general", "git-user")
            .context("git user not specified")?),
        &(config
            .get("general", "git-email")
            .context("git email not specified")?),
    )?;
    let mut git = Git::init(&repo_path)?;
    for mut playlist in
        config
            .sections()
            .into_iter()
            .filter_map(|section| match config.get(&section, "playlist") {
                Some(url) => Playlist::new(
                    section,
                    url,
                    repo_path.clone(),
                    config
                        .get("general", "youtube-dl-path")
                        .unwrap_or_else(|| String::from("youtube-dl")),
                )
                .ok(),
                None => None,
            })
    {
        playlist.download()?;
        playlist.save()?;
    }
    git.add_all()?;
    let commit_changes = if let Ok(diff) = git.has_diff() {
        diff
    } else {
        true
    };
    if commit_changes {
        git.commit(
            &signature,
            &format!("{}: v{}", env!("CARGO_PKG_NAME"), Utc::now().format("%s")),
        )?;
    }
    Ok(())
}
