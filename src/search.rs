use std::path::PathBuf;

use acap::cos::cosine_distance;
use acap::knn::NearestNeighbors;
use acap::vp::VpTree;
use acap::{Distance, Proximity};
use anyhow::Result;
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

impl Proximity<EmbeddedSentence> for EmbeddedSentence {
    type Distance = f32;

    fn distance(&self, other: &Self) -> Self::Distance {
        cosine_distance(&self.embedding, &other.embedding)
    }
}

impl Proximity<EmbeddedSentence> for Vec<f32> {
    type Distance = f32;

    fn distance(&self, other: &EmbeddedSentence) -> Self::Distance {
        cosine_distance(&self, &other.embedding)
    }
}

pub struct EmbeddingTree<'a> {
    tree: VpTree<EmbeddedSentence>,
    model: Model,
    config: &'a MindmapConfig,
}

impl<'a> EmbeddingTree<'a> {
    pub fn new(corpus: Vec<EmbeddedSentence>, model: Model, config: &'a MindmapConfig) -> Self {
        Self {
            tree: VpTree::balanced(corpus),
            model,
            config,
        }
    }

    pub fn rebuild(&mut self, corpus: Vec<EmbeddedSentence>) {
        self.tree = VpTree::balanced(corpus);
    }

    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let emb = self.model.encode(query)?;
        let num_resuls = self.config.num_results;

        let results = self
            .tree
            .k_nearest(&emb, num_resuls)
            .iter()
            .map(|x| SearchResult {
                path: x.item.path.clone(),
                start_line_no: x.item.start_line_no,
                end_line_no: x.item.end_line_no,
                distance: x.distance.value(),
            })
            .collect();

        Ok(results)
    }
}

pub fn search(query: &str, config: &MindmapConfig, formatter: &Formatter) -> Result<()> {
    let corpus = database::get_all(config)?;
    let model = Model::new(config).unwrap();

    let tree = EmbeddingTree::new(corpus, model, config);
    let results = tree.search(query)?;

    // Format response
    let formatted = formatter.format(&results);
    println!("{}", formatted);
    Ok(())
}
