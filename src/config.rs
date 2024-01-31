use anyhow::Result;
use config::{Config, File, FileFormat};
use serde::{Deserialize, Serialize};
use std::{env, fs::OpenOptions, io::Write, path::PathBuf};

use crate::embeddings::ModelType;

#[derive(Debug, Deserialize, Serialize)]
pub struct MindmapConfig {
    pub data_dir: PathBuf,
    pub db_path: PathBuf,
    pub log_path: PathBuf,
    pub lock_path: PathBuf,
    pub min_score: f32,
    pub model: ModelType,
    pub topk: usize,
}

impl MindmapConfig {
    pub fn get_home_dir() -> Option<PathBuf> {
        let home = env::var_os("HOME")?;
        let path = PathBuf::from(home);
        Some(path)
    }

    pub fn get_config_dir() -> Option<PathBuf> {
        let home = Self::get_home_dir()?;
        let config = home.join(".config/");
        let config_dir = config.join("mindmap");
        Some(config_dir)
    }

    pub fn default() -> Self {
        let home = Self::get_home_dir().expect("Home directory should exist");
        let config = Self::get_config_dir().expect("Config directory should exist");
        let mindmap_config = Self {
            data_dir: home.join("mindmap"),
            db_path: config.join("mindmap.db"),
            log_path: config.join("mindmap.log"),
            lock_path: home.join(".mindmap.lock"),
            min_score: 0.2,
            model: ModelType::AllMiniLmL12V2,
            topk: 10,
        };
        mindmap_config.save().expect("Config should save");
        mindmap_config
    }

    pub fn try_load() -> Result<Self> {
        let config_dir = Self::get_config_dir().expect("Config directory should exist");
        let config_file = config_dir.join("config.yaml");
        let builder = Config::builder().add_source(File::new(
            config_file
                .to_str()
                .ok_or(anyhow::anyhow!("No config file"))?,
            FileFormat::Yaml,
        ));
        let config = builder.build()?;
        let mindmap_config = config.try_deserialize::<MindmapConfig>()?;
        Ok(mindmap_config)
    }

    pub fn load() -> Self {
        Self::try_load().unwrap_or_else(|_| Self::default())
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = Self::get_config_dir().expect("Config directory should exist");
        let config_file = config_dir.join("config.yaml");
        let yaml_str = serde_yaml::to_string(self)?;

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(config_file)?;
        file.write_all(yaml_str.as_bytes())?;

        Ok(())
    }
}
