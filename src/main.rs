pub mod git;
pub mod playlist;

use crate::git::Git;
use crate::playlist::Playlist;
use anyhow::{Context, Result};
use chrono::Utc;
use configparser::ini::Ini;
use git2::Signature;
use structopt::StructOpt;

#[derive(Debug, Default, StructOpt)]
#[structopt(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
struct Opt {
    #[structopt(short, long, value_name = "FILE", help = "Sets the configuration file")]
    pub config: Option<String>,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let mut config = Ini::new();
    config
        .load(
            &(opt.config.unwrap_or_else(|| {
                String::from(
                    dirs::config_dir()
                        .expect("config dir not found")
                        .join(format!("{}.ini", env!("CARGO_PKG_NAME")))
                        .to_str()
                        .expect("failed get str from path"),
                )
            })),
        )
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
            .filter_map(|section| match config.get(&section, "url") {
                Some(url) => Playlist::new(
                    section.clone(),
                    url,
                    repo_path.clone(),
                    config
                        .get(&section, "dir")
                        .unwrap_or_else(|| String::from(&section)),
                    config
                        .get(&section, "file")
                        .unwrap_or_else(|| String::from("playlist.ini")),
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
