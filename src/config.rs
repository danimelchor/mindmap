use anyhow::Result;
use config::{Config, File, FileFormat};

pub fn get_config() -> Result<Config> {
    let builder = Config::builder().add_source(File::new("config/settings", FileFormat::Yaml));
    let config = builder.build()?;
    Ok(config)
}
