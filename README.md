# MindMap
[![Latest version](https://img.shields.io/badge/crates.io-v0.1.2-orange)](https://crates.io/crates/mindmap)
[![Documentation](https://img.shields.io/badge/docs-passing-green)](https://docs.rs/mindmap)
![License](https://img.shields.io/badge/License-MIT-blue)

A fast and efficient semantic search engine built in Rust to generate and search embeddings for your notes

## Showcase

### Note watching
<img width="669" alt="image" src="https://github.com/danimelchor/mindmap/assets/24496843/6d95d54e-2ff7-477e-bdff-994db54bd9f2">

### Server querying
```
curl -G --data-urlencode "q=poison" http://127.0.0.1:5001/
curl -G --data-urlencode "q=poison" --data-urlencode "format=json" http://127.0.0.1:5001/
```

<img width="1705" alt="image" src="https://github.com/danimelchor/mindmap/assets/24496843/2fd3ccde-4bd9-4542-b133-b6c353136fbd">

### Pure CLI usage
<img width="693" alt="image" src="https://github.com/danimelchor/mindmap/assets/24496843/d18b45c6-e7a9-42ee-8572-df115dac7bdd">

## Installation

```bash
cargo install mindmap
```

## Usage

```
$ mindmap --help
Search your notes at the speed of thought

Usage: mindmap <COMMAND>

Commands:
  setup           Initial config setup
  watch           Watches your MindMap directory for changes
  recompute-all   Recomputes your entire MindMap
  recompute-file  Recomputes a specific file
  query           Queries the MindMap for items
  server          Starts the MindMap server
  help            Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Setup (Advanced)

MindMap will choose a sane default for the configuration file, but you can also
create it manually. We've created a utility command to help you with that. Just
run `mindmap setup` and it will create the configuration file for you.

```bash
$ mindmap setup
```

Feel free to hit `enter` to select the default values for the config options
that you aren't sure about.


## Models

There are a few models to choose from. The models are:
- AllDistilrobertaV1
- AllMiniLmL12V2
- AllMiniLmL6V2
- BertBaseNliMeanTokens
- DistiluseBaseMultilingualCased
- ParaphraseAlbertSmallV2
- SentenceT5Base

## Server

Despite supporting individual queries with `midnmap query`, MindMap also supports a server mode. The server mode is useful for integrating MindMap with other tools, and it's
also much faster than the CLI mode since it will load the model only once and keep it in memory. To use the server, simply run `mindmap server` and it will start listening on
the address specified in the configuration file:
```
$ mindmap server
Starting server at: 127.0.0.1:5001
```

Then, to communicate with the server just make a GET request to the root path with a query parameter `q`:
```
$ curl -G --data-urlencode "q=are boats cool?" http://127.0.0.1:5001/
~/mindmap/test.md:5:5
~/mindmap/test.md:3:3
~/mindmap/test.md:1:1
~/mindmap/other_data.md:16:17
~/mindmap/other_data.md:48:48
~/mindmap/other_data.md:59:64
~/mindmap/other_data.md:43:43
~/mindmap/other_data.md:31:32
~/mindmap/other_data.md:41:41
~/mindmap/other_data.md:57:57
```

The list of files returned will be in the format `file_path:line_number:column_number`.

## Recommended additional tools

[mindmap.nvim](https://github.com/danimelchor/mindmap.nvim) is a Neovim plugin that allows you to quickly search and edit your MindMap notes.
