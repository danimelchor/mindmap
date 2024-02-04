use crate::config::{MindmapConfig, ModelConfig, ServerConfig};
use anyhow::Result;
use colored::Colorize;
use std::fmt::Debug;
use std::{io::Write, path::PathBuf, str::FromStr};

fn _prompt<T: Debug + FromStr + Clone>(question: &str, default: &T, warning: bool) -> Result<T> {
    let mut input = String::new();
    if warning {
        print!(
            "{} {}{:?}{}: ",
            question.yellow(),
            "[".yellow(),
            default,
            "]".yellow()
        );
    } else {
        print!(
            "{} {}{:?}{}: ",
            question.blue(),
            "[".blue(),
            default,
            "]".blue()
        );
    }

    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut input).unwrap();

    input = input.trim().to_string();
    if input.is_empty() {
        return Ok(default.clone());
    }

    input.parse::<T>().or_else(|_| {
        println!("Invalid input: {:?}", input);
        prompt(question, default)
    })
}

fn prompt<T: Debug + FromStr + Clone>(question: &str, default: &T) -> Result<T> {
    _prompt(question, default, false)
}

fn download_model(model_config: &ModelConfig) -> Result<()> {
    let repo = model_config.model.to_repo();
    println!("> Downloading model from: {}", repo);

    let dir = model_config.get_model_path();
    if dir.exists() {
        let overwrite = _prompt(
            "> Model already exists. Do you want to overwrite it?",
            &false,
            true,
        )?;
        if !overwrite {
            return Ok(());
        }
        std::fs::remove_dir_all(&dir)?;
    }

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

    let data_dir = prompt(
        "Where do you want to write your notes?",
        &def_config.data_dir,
    )?;
    let db_path = prompt(
        "Where do you want to store the database?",
        &def_config.db_path,
    )?;
    let log_path = prompt(
        "Where do you want to store MindMap's logs?",
        &def_config.log_path,
    )?;
    let lock_path = prompt(
        "Where do you want to store MindMap's lock file?",
        &def_config.lock_path,
    )?;
    let min_score = prompt(
        "What is the minimum score to consider a search result?",
        &def_config.min_score,
    )?;

    // Model config
    let model = prompt("What model do you want to use?", &def_config.model.model)?;
    let remote = prompt(
        "Do you want to use a remote model?",
        &def_config.model.remote,
    )?;
    let model_dir = match remote {
        true => PathBuf::new(),
        false => prompt(
            "Where do you want to store the model?",
            &def_config.model.dir,
        )?,
    };
    let model = ModelConfig {
        model,
        remote,
        dir: model_dir,
    };

    // Download model for user
    if !remote {
        let should_download = prompt("Do you want us automatically download the model?", &true)?;
        if should_download {
            download_model(&model)?;
        }
    }

    let num_results = prompt(
        "How many notes do you want to show in search results?",
        &def_config.num_results,
    )?;
    let server = ServerConfig {
        host: prompt("What host do you want to use?", &def_config.server.host)?,
        port: prompt("What port do you want to use?", &def_config.server.port)?,
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
