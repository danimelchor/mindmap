use std::path::PathBuf;

use mindmap::{config::MindmapConfig, database, embeddings::Model, files};

use clap::{Parser, Subcommand};

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
    },

    /// Prints the embeddings for a sentence
    Embed {
        /// The string to embed
        sentence: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = MindmapConfig::load();
    database::start(&config)?;

    match &cli.command {
        Command::Watch => {
            println!("Watching files...");
        }
        Command::RecomputeAll => {
            println!("Recomputing all files...");
            files::recompute_all(&config)?;
        }
        Command::RecomputeFile { file } => {
            println!("Recomputing file: {:?}", file);
            files::recompute_file(file, &config)?;
        }
        Command::Query { query } => {
            println!("Querying for: {:?}", query)
        }
        Command::Embed { sentence } => {
            let model = Model::new(&config.model)?;
            let emb = model.encode(sentence)?;
            print!("{:?}", emb)
        }
    }

    Ok(())
}
