use clap::{Parser, Subcommand};
use colored::Colorize;
use log::LevelFilter;
use mindmap::{
    config::{get_render_config, MindmapConfig},
    database, files,
    formatter::{Formatter, OutputFormat},
    search,
    server::Server,
    setup,
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
    /// Initial config setup
    Setup,

    /// Watches your MindMap directory for changes
    Watch,

    /// Recomputes your entire MindMap
    RecomputeAll {
        /// Skip confirmation
        #[arg(short, long, action)]
        yes: bool,
    },

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
    inquire::set_global_render_config(get_render_config());

    log::info!("Connecting to database");
    database::start(&config)?;

    match cli.command {
        Command::Setup => setup::setup()?,
        Command::Watch => {
            log::info!("Starting watcher");
            let mut mm_watcher = MindmapWatcher::new(config);
            mm_watcher.watch()?;
        }
        Command::RecomputeAll { yes } => {
            let mut confirmed = true;
            if !yes {
                confirmed = inquire::Confirm::new("Are you sure you want to recompute all files?")
                    .with_default(false)
                    .prompt()?;
            }

            if !confirmed {
                log::info!("Aborting recompute all");
                println!("{}", "Aborting recompute all".red());
                return Ok(());
            }

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
