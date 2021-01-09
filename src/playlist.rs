use anyhow::Result;
use configparser::ini::Ini;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Playlist {
    pub name: String,
    pub url: String,
    pub path: PathBuf,
    pub config: Ini,
}

impl Playlist {
    pub fn new(name: String, url: String, repository: String) -> Result<Self> {
        let path = Path::new(&repository).join(&name);
        fs::create_dir_all(&path)?;
        fs::create_dir_all(&path.join("playlist.ini"))?;
        Ok(Self {
            name,
            url,
            path,
            config: Ini::new(),
        })
    }
}
