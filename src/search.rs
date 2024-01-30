use std::path::PathBuf;

use kd_tree::KdPoint;

use crate::{config::MindmapConfig, database::EmbeddedSentence, embeddings::Model};

impl KdPoint for EmbeddedSentence {
    type Scalar = f32;
    type Dim = typenum::U2; // 2 dimensional tree.
    fn at(&self, k: usize) -> f32 {
        self.embedding[k]
    }
}

pub struct Tree {
    kd_tree: Box<kd_tree::KdTree<EmbeddedSentence>>,
    config: MindmapConfig,
}

impl Tree {
    pub fn new(corpus: Vec<EmbeddedSentence>, config: MindmapConfig) -> Self {
        let kd_tree = kd_tree::KdTree::build_by(corpus, |item1, item2, k| {
            item1.embedding[k].partial_cmp(&item2.embedding[k]).unwrap()
        });
        Tree {
            kd_tree: Box::new(kd_tree),
            config,
        }
    }

    pub fn search(self, query: &String) {
        let model = Model::new(&self.config.model).unwrap();
        let emb = model.encode(query).expect("Failed to encode query");
        let rich_topic = EmbeddedSentence {
            path: PathBuf::from(""),
            start_line_no: 0,
            end_line_no: 0,
            embedding: emb,
        };
        let nearests = self.kd_tree.nearests(&rich_topic, 10);
        for nearest in nearests {
            println!(
                "{}:{}:{} ({})",
                nearest.item.path.to_str().unwrap(),
                nearest.item.start_line_no,
                nearest.item.end_line_no,
                nearest.squared_distance
            );

            let content = std::fs::read_to_string(&nearest.item.path).unwrap();
            let lines: Vec<&str> = content.split("\n").collect();
            let start = nearest.item.start_line_no - 1;
            let end = nearest.item.end_line_no;
            for i in start..end {
                println!("{:?}", lines[i as usize]);
            }

            println!();
        }
    }
}
