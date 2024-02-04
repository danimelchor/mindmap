use std::path::PathBuf;

use acap::knn::NearestNeighbors;
use acap::vp::VpTree;
use acap::{euclidean_distance, Distance, EuclideanDistance, Proximity};
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
    type Distance = EuclideanDistance<f32>;

    fn distance(&self, other: &Self) -> Self::Distance {
        euclidean_distance(&self.embedding, &other.embedding)
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
        let emb_sent = EmbeddedSentence {
            path: PathBuf::new(),
            start_line_no: 0,
            end_line_no: 0,
            embedding: emb,
        };

        let topk = self.config.topk;

        let results = self
            .tree
            .k_nearest(&emb_sent, topk)
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
