use acap::{cos::angular_distance, Distance};

use crate::{config::MindmapConfig, database::EmbeddedSentence, embeddings::Model};

pub struct Tree {
    config: MindmapConfig,
    corpus: Vec<EmbeddedSentence>,
}

impl Tree {
    pub fn new(corpus: Vec<EmbeddedSentence>, config: MindmapConfig) -> Self {
        Tree { config, corpus }
    }

    pub fn search(self, query: &String) {
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

        // Print
        println!("Nearest sentences:");
        for (x, dist) in nearests.iter().take(10) {
            println!(
                "{}:{}:{} ({})",
                x.path.to_str().unwrap(),
                x.start_line_no,
                x.end_line_no,
                dist.value()
            );

            let content = std::fs::read_to_string(&x.path).unwrap();
            let lines: Vec<&str> = content.split("\n").collect();
            let start = x.start_line_no - 1;
            let end = x.end_line_no;
            for i in start..end {
                println!("{:?}", lines[i as usize]);
            }

            println!();
        }
    }
}
