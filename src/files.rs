use itertools::Itertools;
use markdown::mdast::Node;
use std::path::Path;
use walkdir::WalkDir;

use crate::{
    config::MindmapConfig,
    database::{self, EmbeddedSentence},
    embeddings::Model,
};
use anyhow::{anyhow, Result};
use std::fs;

pub fn recompute_all(config: &MindmapConfig) -> Result<()> {
    let model = Model::new(config)?;
    let walker = WalkDir::new(&config.data_dir)
        .into_iter()
        .filter_map(|e| e.ok());

    for entry in walker {
        if entry.file_type().is_dir() {
            continue;
        }
        let path = entry.into_path();
        if path.extension().unwrap_or_default() != "md" {
            continue;
        }
        println!("Processing {:?}", path);
        let ast = parse_file(&path)?;
        process_and_store_file(&path, config, &ast, &model)?;
    }
    Ok(())
}

pub fn recompute_file(file: &Path, config: &MindmapConfig) -> Result<()> {
    let model = Model::new(config)?;
    let ast = parse_file(file)?;
    process_and_store_file(file, config, &ast, &model)?;
    Ok(())
}

pub fn process_and_store_file(
    file: &Path,
    config: &MindmapConfig,
    ast: &Node,
    model: &Model,
) -> Result<()> {
    // Delete existing data
    database::delete_file(file, config)?;

    // Process 10 blocks at a time
    let iter = compute_file(ast, file, model);
    for chunk in iter.chunks(10).into_iter() {
        let embs: Vec<EmbeddedSentence> = chunk.collect();
        database::insert_many(&embs, config)?;
    }

    Ok(())
}

fn parse_file(path: &Path) -> Result<Node> {
    let content = fs::read_to_string(path)?;
    let opts = markdown::ParseOptions::default();
    let ast = markdown::to_mdast(&content, &opts).map_err(|e| anyhow!(e))?;
    Ok(ast)
}

fn compute_file<'a>(
    ast: &'a Node,
    path: &'a Path,
    model: &'a Model,
) -> impl Iterator<Item = EmbeddedSentence> + 'a {
    ast.children().unwrap().iter().map(|child| {
        let pos = child.position().expect("No position");
        let start = &pos.start.line;
        let end = &pos.end.line;
        let content = child.to_string();
        let emb = model.encode(&content).unwrap();

        EmbeddedSentence {
            path: path.to_path_buf(),
            start_line_no: *start,
            end_line_no: *end,
            embedding: emb,
        }
    })
}

pub fn delete_file(file: &Path, config: &MindmapConfig) -> Result<()> {
    database::delete_file(file, config)?;
    Ok(())
}
