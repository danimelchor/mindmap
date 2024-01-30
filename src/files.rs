use std::path::PathBuf;

use rust_bert::pipelines::sentence_embeddings::Embedding;
use walkdir::WalkDir;

use crate::{
    config::MindmapConfig,
    database::{self, EmbeddedSentence},
    embeddings::Model,
};
use anyhow::Result;
use std::fs;

pub fn recompute_all(config: &MindmapConfig) -> Result<()> {
    let model = Model::new(&config.model)?;
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
            embs.push(EmbeddedSentence {
                path,
                start_line_no: 0,
                end_line_no: 0,
                embedding: emb,
            });
        }
    }
    database::insert_all(&embs, &config)?;
    Ok(())
}

pub fn recompute_file(file: &PathBuf, config: &MindmapConfig) -> Result<Embedding> {
    let model = Model::new(&config.model)?;
    let emb = _compute_file(file, &model)?;
    let emb = EmbeddedSentence {
        path: file.clone(),
        start_line_no: 0,
        end_line_no: 0,
        embedding: emb,
    };
    database::insert(&emb, &config)?;
    Ok(emb.embedding)
}

pub fn _compute_file(file: &PathBuf, model: &Model) -> Result<Embedding> {
    let content = fs::read_to_string(file)?;
    let emb = model.encode(&content)?;
    Ok(emb)
}
