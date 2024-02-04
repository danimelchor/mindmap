use crate::config::{MindmapConfig, ModelConfig, ServerConfig};
use anyhow::Result;
use colored::Colorize;
use std::fmt::Debug;
use std::{io::Write, path::PathBuf, str::FromStr};

pub fn prompt<T: Debug + FromStr + Clone>(question: &str, default: &T) -> Result<T> {
    let mut input = String::new();
    print!("{} [{:?}]: ", question, default);
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

pub fn setup() -> Result<()> {
    println!("{}", "Welcome to the MindMap assistant!".blue());

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
    let local_path = match remote {
        true => PathBuf::new(),
        false => prompt("Where is the local model?", &def_config.model.local_path)?,
    };
    let model = ModelConfig {
        model,
        remote,
        local_path,
    };
    let topk = prompt("How many notes do you want to see?", &def_config.topk)?;
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
        topk,
        server,
    };
    config.save()?;

    println!("{}", "Setup complete!".green());
    Ok(())
}
