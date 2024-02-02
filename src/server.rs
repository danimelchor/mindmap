use crate::{
    config::MindmapConfig,
    database,
    embeddings::Model,
    formatter::Formatter,
    search::{self},
};
use anyhow::Result;
use colored::Colorize;
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};
use url::Url;

fn handle_stream(
    stream: &mut TcpStream,
    model: &Model,
    config: &MindmapConfig,
    formatter: &Formatter,
) -> Result<String> {
    // Read and parse stream HTTP req
    let buf = &mut [0; 1024];
    stream.read(buf)?;
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);
    req.parse(buf)?;

    // Extract path and query params
    let path = req.path.ok_or(anyhow::anyhow!("No path in request"))?;
    let parsed_url = Url::parse(&format!("http://localhost{}", path))?;
    let hash_query: HashMap<_, _> = parsed_url.query_pairs().into_owned().collect();
    let query = hash_query
        .get("q")
        .ok_or(anyhow::anyhow!("No query in request"))?;

    // Process query
    let corpus = database::get_all(&config)?;
    // let embeddings = corpus.iter().map(|x| x.embedding).collect();
    let results = search::encode_and_search(&model, &corpus, &query.to_string(), config.topk);

    // Format response
    let formatted = formatter.format(&results);
    Ok(formatted)
}

pub fn send_response(code: u16, body: &str, stream: &mut TcpStream) -> Result<()> {
    let length = body.len();
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}",
        code, length, body
    );
    stream.write_all(response.as_bytes())?;
    Ok(())
}

pub fn start(config: &MindmapConfig, formatter: &Formatter) -> Result<()> {
    // Get address
    let host = &config.server.host;
    let port = &config.server.port;
    let addr = format!("{}:{}", host, port);
    log::info!("Starting server at {}", addr);

    // Load model
    log::info!("Loading model: {:?}", config.model);
    println!("{}: {:?}", "Loading model".blue(), &config.model);
    let model = Model::new(&config.model)?;

    // Start app
    log::info!("Starting server at {}", addr);
    println!("{}: {}", "Starting server at".blue(), addr);
    let listener = TcpListener::bind(addr)?;
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let res = handle_stream(&mut stream, &model, &config, &formatter);
        match res {
            Ok(msg) => send_response(200, &msg, &mut stream)?,
            Err(e) => send_response(500, &e.to_string(), &mut stream)?,
        };
    }

    Ok(())
}
