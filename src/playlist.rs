use anyhow::{bail, Result};
use configparser::ini::Ini;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;
use youtube_dl::model::Playlist as YoutubePlaylist;
use youtube_dl::{YoutubeDl, YoutubeDlOutput};

#[derive(Clone, Debug)]
pub struct Playlist {
    pub name: String,
    pub url: String,
    pub path: PathBuf,
    pub config: Ini,
    pub yt_dl_path: String,
    pub yt_playlist: YoutubePlaylist,
}

impl Playlist {
    pub fn new(name: String, url: String, repository: String, yt_dl_path: String) -> Result<Self> {
        let mut config = Ini::new();
        let path = Path::new(&repository).join(&name);
        fs::create_dir_all(&path)?;
        let playlist_file = &path.join("playlist.ini");
        if playlist_file.exists() {
            config
                .load(playlist_file.to_str().expect("failed get str from path"))
                .expect("failed to load configuration file");
        } else {
            File::create(playlist_file)?;
        }
        Ok(Self {
            path,
            config,
            yt_playlist: *match YoutubeDl::new(&url)
                .youtube_dl_path(&yt_dl_path)
                .socket_timeout("15")
                .run()?
            {
                YoutubeDlOutput::SingleVideo(_) => bail!("{} is not a playlist", name),
                YoutubeDlOutput::Playlist(v) => v,
            },
            yt_dl_path,
            name,
            url,
        })
    }

    pub fn save(&mut self) -> Result<()> {
        for video in self.yt_playlist.entries.as_ref().unwrap() {
            self.config.set("contents", &video.title, None);
        }
        self.config.write(
            self.path
                .join("playlist.ini")
                .to_str()
                .expect("failed get str from path"),
        )?;
        Ok(())
    }

    pub fn download(&self) -> Result<()> {
        Command::new(&self.yt_dl_path)
            .args(&[
                &self.url,
                "--extract-audio",
                "--audio-format",
                "mp3",
                "--output",
                &format!(
                    "{}",
                    self.path
                        .join("%(title)s.%(ext)s")
                        .to_str()
                        .expect("failed get str from path")
                ),
            ])
            .spawn()?
            .wait()?;
        Ok(())
    }
}
