use crate::config::{MindmapConfig, ModelConfig, ServerConfig};
use crate::embeddings::ModelType;
use anyhow::Result;
use colored::Colorize;
use inquire::{Confirm, CustomType, Select, Text};
use std::path::PathBuf;

fn download_model(model_config: &ModelConfig) -> Result<()> {
    let repo = model_config.model.to_repo();

    let dir = model_config.get_model_path();
    if dir.exists() {
        let overwrite = Confirm::new("Model already exists. Do you want to overwrite it?")
            .with_default(false)
            .prompt()?;
        if !overwrite {
            return Ok(());
        }
        std::fs::remove_dir_all(&dir)?;
    }

    println!("> Downloading model from: {}", repo);
    let status = std::process::Command::new("git")
        .arg("clone")
        .arg(repo)
        .arg(dir)
        .status()?;

    if !status.success() {
        anyhow::bail!("Failed to download model: {:?}", status);
    }

    Ok(())
}

pub fn setup() -> Result<()> {
    println!("{}", "Welcome to the MindMap assistant!".green());

    let def_config = MindmapConfig::default();

    let data_dir: PathBuf = Text::new("Where do you want to write your notes?")
        .with_default(def_config.data_dir.to_str().unwrap())
        .prompt()?
        .into();

    let db_path: PathBuf = Text::new("Where do you want to store the database?")
        .with_default(def_config.db_path.to_str().unwrap())
        .prompt()?
        .into();
    let log_path: PathBuf = Text::new("Where do you want to store MindMap's logs?")
        .with_default(def_config.log_path.to_str().unwrap())
        .prompt()?
        .into();
    let lock_path: PathBuf = Text::new("Where do you want to store MindMap's lock file?")
        .with_default(def_config.lock_path.to_str().unwrap())
        .prompt()?
        .into();
    let min_score =
        CustomType::<f32>::new("What is the minimum score for a note to be considered relevant?")
            .with_error_message("Invalid input. Please enter a number.")
            .with_default(def_config.min_score)
            .prompt()?;

    // Model config
    // let model = prompt("What model do you want to use?", &def_config.model.model)?;
    let models = ModelType::all();
    let model = Select::new("What model do you want to use?", models).prompt()?;

    let remote = Confirm::new("Do you want to use a remote model?")
        .with_default(def_config.model.remote)
        .prompt()?;

    let model_dir = match remote {
        true => PathBuf::new(),
        false => Text::new("Where do you want to store the model?")
            .with_default(def_config.model.dir.to_str().unwrap())
            .prompt()?
            .into(),
    };
    let model = ModelConfig {
        model,
        remote,
        dir: model_dir,
    };

    // Download model for user
    if !remote {
        let should_download = Confirm::new("Do you want us automatically download the model?")
            .with_default(true)
            .prompt()?;
        if should_download {
            download_model(&model)?;
        }
    }

    let num_results =
        CustomType::<usize>::new("How many notes do you want to show in search results?")
            .with_error_message("Invalid input. Please enter a number.")
            .with_default(def_config.num_results)
            .prompt()?;
    let server = ServerConfig {
        host: Text::new("What host do you want to use?")
            .with_default(&def_config.server.host)
            .prompt()?,
        port: CustomType::<u16>::new("What port do you want to use?")
            .with_error_message("Invalid input. Please enter a number.")
            .with_default(def_config.server.port)
            .prompt()?,
    };

    let config = MindmapConfig {
        data_dir,
        db_path,
        log_path,
        lock_path,
        min_score,
        model,
        num_results,
        server,
    };
    config.save()?;

    println!("{}", "Setup complete!".green());
    Ok(())
}
