use rusqlite::Connection;
use rust_bert::pipelines::sentence_embeddings::Embedding;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::{config::MindmapConfig, server};

#[derive(Debug)]
pub struct EmbeddedSentence {
    pub path: PathBuf,
    pub start_line_no: usize,
    pub end_line_no: usize,
    pub embedding: Embedding,
}

pub fn start(config: &MindmapConfig) -> Result<()> {
    let conn = Connection::open(&config.db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sentences (
            id INTEGER PRIMARY KEY,
            path TEXT,
            start_line_no INTEGER,
            end_line_no INTEGER,
            embedding BLOB
        )",
        (),
    )?;
    Ok(())
}

fn u8_to_f32(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(TryInto::try_into)
        .map(Result::unwrap)
        .map(f32::from_le_bytes)
        .collect()
}

fn f32_to_u8(floats: &[f32]) -> Vec<u8> {
    floats.iter().flat_map(|f| f.to_le_bytes()).collect()
}

pub fn get_all(config: &MindmapConfig) -> Result<Vec<EmbeddedSentence>> {
    let conn = Connection::open(&config.db_path)?;
    let mut stmt =
        conn.prepare("SELECT path, start_line_no, end_line_no, embedding FROM sentences")?;
    let rows = stmt
        .query_map([], |row| {
            let path = row.get::<_, String>(0)?;
            let start_line_no = row.get::<_, usize>(1)?;
            let end_line_no = row.get::<_, usize>(2)?;
            let embedding = row.get::<_, Vec<u8>>(3)?;

            Ok(EmbeddedSentence {
                path: PathBuf::from(path),
                start_line_no,
                end_line_no,
                embedding: u8_to_f32(&embedding),
            })
        })
        .into_iter()
        .flatten()
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn insert_many(embs: &Vec<EmbeddedSentence>, config: &MindmapConfig) -> Result<()> {
    let mut conn = Connection::open(&config.db_path)?;
    let tx = conn.transaction()?;
    for emb in embs {
        tx.execute(
            "INSERT INTO sentences (path, start_line_no, end_line_no, embedding) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![emb.path.to_str(), emb.start_line_no, emb.end_line_no, f32_to_u8(&emb.embedding)],
        )?;
    }
    tx.commit()?;
    server::notify_rebuild(config).ok();
    Ok(())
}

pub fn delete_all(config: &MindmapConfig) -> Result<()> {
    let conn = Connection::open(&config.db_path)?;
    conn.execute("DELETE FROM sentences", [])?;
    server::notify_rebuild(config).ok();
    Ok(())
}

pub fn delete_file(file: &Path, config: &MindmapConfig) -> Result<()> {
    let conn = Connection::open(&config.db_path)?;
    conn.execute(
        "DELETE FROM sentences WHERE path = ?1",
        rusqlite::params![file.to_str()],
    )?;
    server::notify_rebuild(config).ok();
    Ok(())
}
