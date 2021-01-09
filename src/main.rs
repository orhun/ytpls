pub mod playlist;

use crate::playlist::Playlist;
use anyhow::{Context, Result};
use configparser::ini::Ini;
use youtube_dl::YoutubeDl;

fn main() -> Result<()> {
    let mut config = Ini::new();
    config
        .load("example.ini")
        .expect("failed to load configuration file");
    let repository = config
        .get("general", "repository")
        .context("no repository field")?;
    for playlist in
        config
            .sections()
            .into_iter()
            .filter_map(|section| match config.get(&section, "playlist") {
                Some(url) => Playlist::new(section, url, repository.clone()).ok(),
                None => None,
            })
    {
        println!("{:?}", playlist);
    }
    Ok(())
}
