use crate::search::SearchResult;
use clap::ValueEnum;
use colored::Colorize;
use serde::Serialize;
use std::{fmt::Display, path::PathBuf, str::FromStr};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum OutputFormat {
    /// List format
    List,
    /// Raw format
    Raw,
    /// JSON format
    Json,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "list" => Ok(OutputFormat::List),
            "raw" => Ok(OutputFormat::Raw),
            "json" => Ok(OutputFormat::Json),
            _ => Err("Invalid output format".to_string()),
        }
    }
}

impl Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::List => write!(f, "list"),
            OutputFormat::Raw => write!(f, "raw"),
            OutputFormat::Json => write!(f, "json"),
        }
    }
}

#[derive(Serialize, Debug)]
struct SearchResultWithContext {
    pub path: PathBuf,
    pub start_line_no: usize,
    pub end_line_no: usize,
    pub distance: f32,
    context: String,
}

fn get_context(result: &SearchResult) -> String {
    let start_no = result.start_line_no;
    let end_no = result.end_line_no;

    let content = std::fs::read_to_string(&result.path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();
    let start_no = if start_no > 0 { start_no - 1 } else { 0 };
    let end_no = if end_no < total_lines {
        end_no
    } else {
        total_lines
    };
    lines[start_no..end_no].join("\n")
}

pub fn format(results: &Vec<SearchResult>, format: OutputFormat) -> String {
    let with_context = results
        .iter()
        .map(|r| SearchResultWithContext {
            path: r.path.clone(),
            start_line_no: r.start_line_no,
            end_line_no: r.end_line_no,
            distance: r.distance,
            context: get_context(r),
        })
        .collect();

    match format {
        OutputFormat::List => list(&with_context),
        OutputFormat::Raw => raw(&with_context),
        OutputFormat::Json => json(&with_context),
    }
}

fn list(results: &Vec<SearchResultWithContext>) -> String {
    let mut sentences = vec![];
    for r in results {
        let title = format!(
            "{}:{}:{} - {}",
            r.path.display(),
            r.start_line_no,
            r.end_line_no,
            r.distance
        );
        let sentence = format!("{}\n{}", title.blue(), r.context);
        sentences.push(sentence);
    }
    sentences.join("\n\n")
}

fn raw(results: &Vec<SearchResultWithContext>) -> String {
    let mut fmt = String::new();
    for r in results {
        fmt.push_str(&format!(
            "{}:{}:{}\n",
            r.path.display(),
            r.start_line_no,
            r.end_line_no
        ));
    }
    fmt
}

fn json(results: &Vec<SearchResultWithContext>) -> String {
    serde_json::to_string(results).unwrap()
}
