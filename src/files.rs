use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::{
    config::MindmapConfig,
    database::{self, EmbeddedSentence},
    embeddings::Model,
};
use anyhow::Result;
use std::fs;

pub fn recompute_all(config: &MindmapConfig) -> Result<()> {
    let model = Model::new(config)?;
    let walker = WalkDir::new(&config.data_dir)
        .into_iter()
        .filter_map(|e| e.ok());

    let mut embs = vec![];
    for entry in walker {
        if entry.file_type().is_dir() {
            continue;
        }
        let path = entry.into_path();
        let emb = _compute_file(&path, &model);
        if let Ok(emb) = emb {
            embs.extend(emb);
        }
    }
    database::delete_all(config)?;
    database::insert_many(&embs, config)?;
    Ok(())
}

pub fn recompute_file(file: &PathBuf, config: &MindmapConfig) -> Result<()> {
    let model = Model::new(config)?;
    let emb = _compute_file(file, &model)?;
    database::delete_file(file, config)?;
    database::insert_many(&emb, config)?;
    Ok(())
}

pub fn _compute_file(file: &PathBuf, model: &Model) -> Result<Vec<EmbeddedSentence>> {
    let content = fs::read_to_string(file)?;
    let opts = markdown::ParseOptions::default();
    let ast = markdown::to_mdast(&content, &opts);

    if ast.is_err() {
        return Err(anyhow::anyhow!("Failed to parse file: {:?}", file));
    }
    let ast = ast.unwrap();

    let children = ast.children().ok_or(anyhow::anyhow!("No children"))?;
    let embs = children.iter().map(|child| {
        let pos = child.position().expect("No position");
        let start = &pos.start.line;
        let end = &pos.end.line;
        let content = child.to_string();
        let emb = model.encode(&content)?;

        Ok(EmbeddedSentence {
            path: file.clone(),
            start_line_no: *start,
            end_line_no: *end,
            embedding: emb,
        })
    });

    let embs = embs.collect::<Result<Vec<_>>>()?;
    Ok(embs)
}

pub fn delete_file(file: &Path, config: &MindmapConfig) -> Result<()> {
    database::delete_file(file, config)?;
    Ok(())
}
