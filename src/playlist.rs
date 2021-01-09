use anyhow::{bail, Result};
use configparser::ini::Ini;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Command;
use youtube_dl::model::Playlist as YoutubePlaylist;
use youtube_dl::{YoutubeDl, YoutubeDlOutput};

#[derive(Clone, Debug)]
pub struct Playlist {
    name: String,
    url: String,
    path: PathBuf,
    config: Ini,
    config_file: String,
    yt_dl_path: String,
    yt_playlist: YoutubePlaylist,
}

impl Playlist {
    pub fn new(
        name: String,
        url: String,
        repo_path: String,
        directory: String,
        config_file: String,
        yt_dl_path: String,
    ) -> Result<Self> {
        let mut config = Ini::new_cs();
        let path = Path::new(&repo_path).join(&directory);
        fs::create_dir_all(&path)?;
        let playlist_file = &path.join(&config_file);
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
            config_file,
            yt_dl_path,
            name,
            url,
        })
    }

    pub fn save(&mut self) -> Result<()> {
        for video in self.yt_playlist.entries.as_ref().expect("no entries") {
            self.config.set("contents", &video.title, None);
            self.config
                .set("archive", &format!("youtube {}", &video.id), None);
        }
        self.config.write(
            self.path
                .join(&self.config_file)
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
                "--download-archive",
                self.path
                    .join(&self.config_file)
                    .to_str()
                    .expect("failed get str from path"),
                "--output",
                self.path
                    .join("%(title)s.%(ext)s")
                    .to_str()
                    .expect("failed get str from path"),
            ])
            .spawn()?
            .wait()?;
        Ok(())
    }
}
