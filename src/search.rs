use std::path::PathBuf;

use acap::{cos::angular_distance, Distance};

use crate::{config::MindmapConfig, database::EmbeddedSentence, embeddings::Model};

pub struct Tree {
    config: MindmapConfig,
    corpus: Vec<EmbeddedSentence>,
}

#[derive(Debug)]
pub struct SearchResult {
    pub path: PathBuf,
    pub start_line_no: usize,
    pub end_line_no: usize,
    pub distance: f32,
}

impl Tree {
    pub fn new(corpus: Vec<EmbeddedSentence>, config: MindmapConfig) -> Self {
        Tree { config, corpus }
    }

    pub fn search(self, query: &String) -> Vec<SearchResult> {
        let model = Model::new(&self.config.model).unwrap();
        let emb = model.encode(query).expect("Failed to encode query");

        let corpus_emb = self
            .corpus
            .iter()
            .map(|x| {
                let dist = angular_distance(&emb, &x.embedding);
                (x, dist)
            })
            .collect::<Vec<_>>();

        // Sort
        let mut nearests = corpus_emb;
        nearests.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        nearests
            .iter()
            .map(|(x, dist)| SearchResult {
                path: x.path.clone(),
                start_line_no: x.start_line_no,
                end_line_no: x.end_line_no,
                distance: dist.value(),
            })
            .take(self.config.topk)
            .collect()
    }
}
