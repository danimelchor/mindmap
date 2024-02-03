use clap::{Parser, Subcommand};
use colored::Colorize;
use log::LevelFilter;
use mindmap::{
    config::MindmapConfig,
    database, files,
    formatter::{Formatter, OutputFormat},
    search,
    server::Server,
    watcher::MindmapWatcher,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Watches your MindMap directory for changes
    Watch,

    /// Recomputes your entire MindMap
    RecomputeAll,

    /// Recomputes a specific file
    RecomputeFile {
        /// The file to recompute
        file: PathBuf,
    },

    /// Queries the MindMap for items
    Query {
        /// The idea to search for
        query: String,

        /// The output format
        #[arg(value_enum, short, long, default_value = "list")]
        format: OutputFormat,
    },

    /// Starts the MindMap server
    Server {
        /// The output format
        #[arg(value_enum, short, long, default_value = "raw")]
        format: OutputFormat,
    },
}

fn main() -> anyhow::Result<()> {
    let config = MindmapConfig::load();
    simple_logging::log_to_file(&config.log_path, LevelFilter::Info).unwrap();
    log::debug!("Loaded config");

    let cli = Cli::parse();

    log::info!("Connecting to database");
    database::start(&config)?;

    match cli.command {
        Command::Watch => {
            log::info!("Starting watcher");
            let mut mm_watcher = MindmapWatcher::new(config);
            mm_watcher.watch()?;
        }
        Command::RecomputeAll => {
            log::info!("Recomputing all files");
            println!("{}", "Recomputing all files...".blue());
            files::recompute_all(&config)?;
        }
        Command::RecomputeFile { file } => {
            log::info!("Recomputing file: {:?}", file);
            println!("{}: {:?}", "Recomputing file".blue(), file);
            files::recompute_file(&file, &config)?;
        }
        Command::Query { query, format } => {
            log::info!("Searching for: {}", query);
            let formatter = Formatter::new(format);
            search::search(&query, &config, &formatter)?;
        }
        Command::Server { format } => {
            let formatter = Formatter::new(format);
            Server::start(&config, &formatter)?;
        }
    }

    Ok(())
}
