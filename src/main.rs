pub mod git;
pub mod playlist;

use crate::git::Git;
use crate::playlist::Playlist;
use anyhow::{Context, Result};
use chrono::Utc;
use configparser::ini::Ini;
use git2::Signature;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use structopt::StructOpt;

#[derive(Debug, Default, StructOpt)]
#[structopt(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
struct Opt {
    #[structopt(short, long, help = "Activates the debug mode")]
    debug: bool,
    #[structopt(short, long, value_name = "FILE", help = "Sets the configuration file")]
    pub config: Option<String>,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    SimpleLogger::new()
        .with_level(if opt.debug {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .init()?;
    let mut config = Ini::new();
    let config_file = &(opt.config.unwrap_or_else(|| {
        String::from(
            dirs::config_dir()
                .expect("config dir not found")
                .join(format!("{}.ini", env!("CARGO_PKG_NAME")))
                .to_str()
                .expect("failed get str from path"),
        )
    }));
    log::debug!("Using configuration file: {:?}", config_file);
    config
        .load(config_file)
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
    log::debug!("Repository: {:?}", repo_path);
    let mut git = Git::init(&repo_path)?;
    for mut playlist in
        config
            .sections()
            .into_iter()
            .filter_map(|section| match config.get(&section, "url") {
                Some(url) => {
                    log::info!("Fetching \"{}\"...", section);
                    log::debug!("Playlist URL: {:?}", url);
                    Playlist::new(
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
                        config
                            .get("general", "socket-timeout")
                            .unwrap_or_default()
                            .parse()
                            .unwrap_or(15),
                    )
                    .ok()
                }
                None => None,
            })
    {
        log::info!(
            "Downloading {} tracks from \"{}\"",
            playlist
                .yt_playlist
                .entries
                .as_ref()
                .expect("no entries found")
                .len(),
            playlist.name,
        );
        log::debug!(
            "Playlist \"{}\" is uploaded by \"{}\"",
            playlist
                .yt_playlist
                .title
                .as_ref()
                .map_or("", |v| v.as_ref()),
            playlist
                .yt_playlist
                .uploader
                .as_ref()
                .map_or("", |v| v.as_ref())
        );
        playlist.download()?;
        log::info!("Saving the tracks...");
        log::debug!("Archive file: {:?}", playlist.config_file);
        playlist.save()?;
    }
    git.add_all()?;
    let commit_changes = if let Ok(diff) = git.has_diff() {
        diff
    } else {
        true
    };
    if commit_changes {
        log::info!("Committing the changes...");
        git.commit(
            &signature,
            &format!("{}: v{}", env!("CARGO_PKG_NAME"), Utc::now().format("%s")),
        )?;
    } else {
        log::info!("There's nothing to commit");
    }
    log::info!("Done!");
    Ok(())
}
