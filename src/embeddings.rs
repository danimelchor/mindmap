use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use anyhow::Result;
use rust_bert::pipelines::sentence_embeddings::{
    Embedding, SentenceEmbeddingsBuilder, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
};
use serde::{Deserialize, Serialize};

use crate::config::MindmapConfig;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ModelType {
    AllMiniLmL12V2,
    AllMiniLmL6V2,
    AllDistilrobertaV1,
    BertBaseNliMeanTokens,
    DistiluseBaseMultilingualCased,
    ParaphraseAlbertSmallV2,
    SentenceT5Base,
}

impl ModelType {
    pub fn all() -> Vec<ModelType> {
        vec![
            ModelType::AllMiniLmL12V2,
            ModelType::AllMiniLmL6V2,
            ModelType::AllDistilrobertaV1,
            ModelType::BertBaseNliMeanTokens,
            ModelType::DistiluseBaseMultilingualCased,
            ModelType::ParaphraseAlbertSmallV2,
            ModelType::SentenceT5Base,
        ]
    }

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

    pub fn to_repo_name(&self) -> &str {
        match self {
            ModelType::BertBaseNliMeanTokens => "bert-base-nli-mean-tokens",
            ModelType::DistiluseBaseMultilingualCased => "distiluse-base-multilingual-cased",
            ModelType::AllMiniLmL12V2 => "all-MiniLM-L12-v2",
            ModelType::AllMiniLmL6V2 => "all-MiniLM-L6-v2",
            ModelType::AllDistilrobertaV1 => "all-distilroberta-v1",
            ModelType::ParaphraseAlbertSmallV2 => "paraphrase-albert-small-v2",
            ModelType::SentenceT5Base => "sentence-T5-base",
        }
    }

    pub fn to_repo(&self) -> String {
        let base = "https://huggingface.co/sentence-transformers/";
        let repo = self.to_repo_name();
        format!("{}{}", base, repo)
    }
}

impl FromStr for ModelType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BertBaseNliMeanTokens" => Ok(ModelType::BertBaseNliMeanTokens),
            "DistiluseBaseMultilingualCased" => Ok(ModelType::DistiluseBaseMultilingualCased),
            "AllMiniLmL12V2" => Ok(ModelType::AllMiniLmL12V2),
            "AllMiniLmL6V2" => Ok(ModelType::AllMiniLmL6V2),
            "AllDistilrobertaV1" => Ok(ModelType::AllDistilrobertaV1),
            "ParaphraseAlbertSmallV2" => Ok(ModelType::ParaphraseAlbertSmallV2),
            "SentenceT5Base" => Ok(ModelType::SentenceT5Base),
            _ => Err(()),
        }
    }
}

impl Display for ModelType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = self.to_repo_name();
        write!(f, "{}", s)
    }
}

pub struct Model {
    model: SentenceEmbeddingsModel,
}

impl Model {
    pub fn new(config: &MindmapConfig) -> Result<Self> {
        let model_config = &config.model;
        let rust_bert_type = model_config.model.to_rust_bert();
        let model = match model_config.remote {
            true => SentenceEmbeddingsBuilder::remote(rust_bert_type)
                .with_device(tch::Device::cuda_if_available())
                .create_model()?,
            false => SentenceEmbeddingsBuilder::local(model_config.get_model_path())
                .with_device(tch::Device::cuda_if_available())
                .create_model()?,
        };
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
