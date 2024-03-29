use anyhow::Result;
use config::{Config, File, FileFormat};
use inquire::ui::{Color, RenderConfig, StyleSheet, Styled};
use serde::{Deserialize, Serialize};
use std::{env, fs::OpenOptions, io::Write, path::PathBuf};

use crate::embeddings::ModelType;

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub lock_path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WatcherConfig {
    pub lock_path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModelConfig {
    pub model: ModelType,
    pub remote: bool,
    pub dir: PathBuf,
}

impl ModelConfig {
    pub fn get_model_path(&self) -> PathBuf {
        let dir = &self.dir;
        let repo_name = self.model.to_repo_name();
        dir.join(repo_name)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MindmapConfig {
    pub data_dir: PathBuf,
    pub db_path: PathBuf,
    pub log_path: PathBuf,
    pub min_score: f32,
    pub num_results: usize,
    pub server: ServerConfig,
    pub model: ModelConfig,
    pub watcher: WatcherConfig,
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

    pub fn try_load() -> Result<Self> {
        let config_dir =
            Self::get_config_dir().ok_or(anyhow::anyhow!("Config directory should exist"))?;
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

impl Default for MindmapConfig {
    fn default() -> Self {
        let home = Self::get_home_dir().expect("Home directory should exist");
        let config = Self::get_config_dir().expect("Config directory should exist");
        let model = ModelType::AllMiniLmL12V2;
        let mindmap_config = Self {
            data_dir: home.join("notes"),
            db_path: config.join("mindmap.db"),
            log_path: config.join("mindmap.log"),
            min_score: 0.25,
            model: ModelConfig {
                model,
                remote: true,
                dir: config.join("models/"),
            },
            num_results: 20,
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 5001,
                lock_path: home.join(".mindmap-server.lock"),
            },
            watcher: WatcherConfig {
                lock_path: home.join(".mindmap-watcher.lock"),
            },
        };
        mindmap_config.save().expect("Config should save");
        mindmap_config
    }
}

pub fn get_render_config() -> RenderConfig {
    let mut render_config = RenderConfig::default();
    render_config.prompt_prefix = Styled::new(">").with_fg(Color::LightBlue);
    render_config.answered_prompt_prefix = Styled::new(">").with_fg(Color::LightCyan);
    render_config.error_message = render_config
        .error_message
        .with_prefix(Styled::new("❌").with_fg(Color::LightRed));

    render_config.prompt = StyleSheet::new().with_fg(Color::LightBlue);
    render_config
}
