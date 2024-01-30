use anyhow::Result;
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
};

pub struct Model {
    model: SentenceEmbeddingsModel,
}

impl Model {
    pub fn new() -> Result<Self> {
        let model = SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL12V2)
            .create_model()?;
        Ok(Self { model })
    }

    pub fn encode_many(self, sentences: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        let embeddings = self.model.encode(&sentences)?;
        Ok(embeddings)
    }

    pub fn encode(self, text: &str) -> Result<Vec<f32>> {
        let output = self.encode_many(vec![text])?;
        Ok(output[0].clone())
    }
}
