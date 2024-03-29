use crate::{
    config::MindmapConfig,
    database,
    embeddings::Model,
    formatter::{self, OutputFormat},
    search::EmbeddingTree,
    utils,
};
use anyhow::Result;
use colored::Colorize;
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};
use url::Url;

pub fn notify_rebuild(config: &MindmapConfig) -> Result<()> {
    let host = &config.server.host;
    let port = config.server.port;
    let addr = format!("{}:{}", host, port);

    let get = format!("GET /rebuild HTTP/1.1\r\nHost: {}\r\n\r\n", addr);

    let mut sock = TcpStream::connect(addr)?;
    sock.write_all(get.as_bytes())?;

    Ok(())
}

enum RequestType {
    Search(String, OutputFormat),
    Rebuild,
}

pub struct Server;

impl Server {
    fn read_full(stream: &mut TcpStream) -> Result<Vec<u8>> {
        let mut buf = vec![0; 1024];
        let mut total = Vec::new();
        loop {
            let read = stream.read(&mut buf)?;
            total.extend_from_slice(&buf[..read]);
            if read < buf.len() {
                break;
            }
        }
        Ok(total)
    }

    fn parse_request(stream: &mut TcpStream) -> Result<RequestType> {
        // Read and parse stream HTTP req
        let buf = Self::read_full(stream)?;

        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);
        req.parse(&buf)?;

        // Extract path and query params
        let path = req.path.ok_or(anyhow::anyhow!("No path in request"))?;
        if path == "/rebuild" {
            return Ok(RequestType::Rebuild);
        }

        let parsed_url = Url::parse(&format!("http://localhost{}", path))?;
        let hash_query: HashMap<_, _> = parsed_url.query_pairs().into_owned().collect();
        let query = hash_query
            .get("q")
            .ok_or(anyhow::anyhow!("No query in request"))?;

        let output_format = hash_query
            .get("format")
            .map(|f| f.parse().unwrap_or(OutputFormat::Raw))
            .unwrap_or(OutputFormat::Raw);

        Ok(RequestType::Search(query.to_string(), output_format))
    }

    fn handle_query(query: &String, format: OutputFormat, tree: &EmbeddingTree) -> Result<String> {
        println!("{} '{}' ({})", "Querying for".blue(), query, format);
        let results = tree.search(&query.to_string())?;
        let formatted = formatter::format(&results, format);
        Ok(formatted)
    }

    fn handle_rebuild(tree: &mut EmbeddingTree, config: &MindmapConfig) -> Result<String> {
        let corpus = database::get_all(config)?;
        println!("{}", "Rebuilding...".blue());
        tree.rebuild(corpus);
        Ok("Rebuilt".to_string())
    }

    fn send_response(code: u16, body: &str, stream: &mut TcpStream) -> Result<()> {
        let length = body.len();

        // CORS stuff
        let headers =
            "Access-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET\r\n".to_string();

        let response = format!(
            "HTTP/1.1 {}\r\n{}Content-Length: {}\r\n\r\n{}",
            code, headers, length, body
        );
        stream.write_all(response.as_bytes())?;
        Ok(())
    }

    pub fn start(config: &MindmapConfig) -> Result<()> {
        utils::acquire_lock(&config.server.lock_path)?;

        // Get address
        let host = &config.server.host;
        let port = &config.server.port;
        let addr = format!("{}:{}", host, port);
        log::info!("Starting server at {}", addr);

        // Load model
        log::info!("Loading model: {:?}", config.model);
        println!("{}: {:?}", "Loading model".blue(), &config.model);
        let model = Model::new(config)?;
        let corpus = database::get_all(config)?;
        let mut tree = EmbeddingTree::new(corpus, model, config);

        // Start app
        log::info!("Starting server at {}", addr);
        println!("{}: {}", "Starting server at".blue(), addr);
        let listener = TcpListener::bind(addr)?;

        for stream in listener.incoming() {
            let mut stream = stream.unwrap();

            // Parse stream
            let stream_type = Self::parse_request(&mut stream);
            if let Err(err) = stream_type {
                Self::send_response(400, &err.to_string(), &mut stream)?;
                continue;
            }

            let res = match stream_type.unwrap() {
                RequestType::Search(query, format) => Self::handle_query(&query, format, &tree),
                RequestType::Rebuild => Self::handle_rebuild(&mut tree, config),
            };

            // Send response
            match res {
                Ok(msg) => Self::send_response(200, &msg, &mut stream)?,
                Err(e) => Self::send_response(500, &e.to_string(), &mut stream)?,
            };
        }

        Ok(())
    }
}
