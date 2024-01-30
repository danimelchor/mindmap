use anyhow::Result;
use rust_bert::pipelines::sentence_embeddings::{
    Embedding, SentenceEmbeddingsBuilder, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ModelType {
    BertBaseNliMeanTokens,
    DistiluseBaseMultilingualCased,
    AllMiniLmL12V2,
    AllMiniLmL6V2,
    AllDistilrobertaV1,
    ParaphraseAlbertSmallV2,
    SentenceT5Base,
}

impl ModelType {
    pub fn to_rust_bert(&self) -> SentenceEmbeddingsModelType {
        match self {
            ModelType::BertBaseNliMeanTokens => SentenceEmbeddingsModelType::BertBaseNliMeanTokens,
            ModelType::DistiluseBaseMultilingualCased => {
                SentenceEmbeddingsModelType::DistiluseBaseMultilingualCased
            }
            ModelType::AllMiniLmL12V2 => SentenceEmbeddingsModelType::AllMiniLmL12V2,
            ModelType::AllMiniLmL6V2 => SentenceEmbeddingsModelType::AllMiniLmL6V2,
            ModelType::AllDistilrobertaV1 => SentenceEmbeddingsModelType::AllDistilrobertaV1,
            ModelType::ParaphraseAlbertSmallV2 => {
                SentenceEmbeddingsModelType::ParaphraseAlbertSmallV2
            }
            ModelType::SentenceT5Base => SentenceEmbeddingsModelType::SentenceT5Base,
        }
    }

    pub fn from_rust_bert(model: SentenceEmbeddingsModelType) -> Self {
        match model {
            SentenceEmbeddingsModelType::BertBaseNliMeanTokens => ModelType::BertBaseNliMeanTokens,
            SentenceEmbeddingsModelType::DistiluseBaseMultilingualCased => {
                ModelType::DistiluseBaseMultilingualCased
            }
            SentenceEmbeddingsModelType::AllMiniLmL12V2 => ModelType::AllMiniLmL12V2,
            SentenceEmbeddingsModelType::AllMiniLmL6V2 => ModelType::AllMiniLmL6V2,
            SentenceEmbeddingsModelType::AllDistilrobertaV1 => ModelType::AllDistilrobertaV1,
            SentenceEmbeddingsModelType::ParaphraseAlbertSmallV2 => {
                ModelType::ParaphraseAlbertSmallV2
            }
            SentenceEmbeddingsModelType::SentenceT5Base => ModelType::SentenceT5Base,
        }
    }
}

pub struct Model {
    model: SentenceEmbeddingsModel,
}

impl Model {
    pub fn new(model_type: &ModelType) -> Result<Self> {
        let rust_bert_type = model_type.to_rust_bert();
        let model = SentenceEmbeddingsBuilder::remote(rust_bert_type).create_model()?;
        Ok(Self { model })
    }

    pub fn encode_many(&self, sentences: Vec<&str>) -> Result<Vec<Embedding>> {
        let embeddings = self.model.encode(&sentences)?;
        Ok(embeddings)
    }

    pub fn encode(&self, text: &str) -> Result<Embedding> {
        let output = self.encode_many(vec![text])?;
        Ok(output[0].clone())
    }
}
