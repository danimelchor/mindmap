use std::path::PathBuf;

use acap::{
    cos::{angular_distance, cosine_distance},
    Distance,
};
use anyhow::Result;
use rust_bert::pipelines::sentence_embeddings::Embedding;
use serde::Serialize;

use crate::{
    config::MindmapConfig,
    database::{self, EmbeddedSentence},
    embeddings::Model,
    formatter::Formatter,
};

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub path: PathBuf,
    pub start_line_no: usize,
    pub end_line_no: usize,
    pub distance: f32,
}

pub fn encode_and_search(
    model: &Model,
    corpus: &Vec<EmbeddedSentence>,
    query: &String,
    topk: usize,
) -> Vec<SearchResult> {
    let emb = model.encode(query).expect("Failed to encode query");

    // let cos_sims = cosine_distance(&emb, &corpus);

    let mut cos_sims = corpus
        .iter()
        .map(|x| {
            let dist = angular_distance(&emb, &x.embedding);
            (x, dist)
        })
        .collect::<Vec<_>>();

    // Sort
    cos_sims.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    cos_sims
        .into_iter()
        .map(|(x, dist)| SearchResult {
            path: x.path.clone(),
            start_line_no: x.start_line_no,
            end_line_no: x.end_line_no,
            distance: dist.value(),
        })
        .take(topk)
        .collect()
}

pub fn search(query: &String, config: &MindmapConfig, formatter: &Formatter) -> Result<()> {
    let corpus = database::get_all(config)?;
    // TODO: optimize cos similarity
    // let embeddings = corpus.iter().map(|x| x.embedding).collect::<Vec<_>>();

    let model = Model::new(&config.model).unwrap();
    let results = encode_and_search(&model, &corpus, query, config.topk);

    // Format response
    let formatted = formatter.format(&results);
    println!("{}", formatted);
    Ok(())
}
