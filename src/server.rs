use crate::{
    config::MindmapConfig,
    database,
    embeddings::Model,
    formatter,
    search::{self},
};
use anyhow::Result;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

fn handle_stream(stream: &mut TcpStream, model: &Model, config: &MindmapConfig) -> Result<String> {
    // Read stream using httparse
    let buf = &mut [0; 1024];
    stream.read(buf)?;
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);
    req.parse(buf)?;

    // Extract path and query params
    let path = req.path.ok_or(anyhow::anyhow!("No path in request"))?;
    let query_str = path
        .split_once("?")
        .ok_or(anyhow::anyhow!("No query in path"))?
        .1;
    let query_params: Vec<(&str, &str)> = query_str
        .split("&")
        .map(|qp| qp.split_once("="))
        .filter_map(|x| x)
        .filter(|(k, _)| *k == "q")
        .collect();

    if query_params.len() != 1 {
        return Err(anyhow::anyhow!("Missing query parameter q"));
    }
    let query = query_params[0].1;
    println!("Searching for '{}'", query);

    // Process query
    let corpus = database::get_all(&config)?;
    let results = search::encode_and_search(&model, &corpus, &query.to_string(), config.topk);

    // Format response
    let formatted = formatter::raw(&results);
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

pub fn start(config: MindmapConfig) -> Result<()> {
    // Get address
    let host = &config.server.host;
    let port = &config.server.port;
    let addr = format!("{}:{}", host, port);

    // Load model
    let model = Model::new(&config.model)?;

    // Start app
    println!("Starting server at: {}", addr);
    let listener = TcpListener::bind(addr)?;
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let res = handle_stream(&mut stream, &model, &config);
        match res {
            Ok(msg) => send_response(200, &msg, &mut stream)?,
            Err(e) => send_response(500, &e.to_string(), &mut stream)?,
        };
    }

    Ok(())
}
