# MindMap

`MindMap` is a simple CLI tool that allows you to search your notes using semantic search. MindMap offers quite a few models to choose from.

## Installation

```bash
cargo install mindmap
```

## Usage

```bash
mindmap --help
Search your notes at the speed of thought

Usage: mindmap <COMMAND>

Commands:
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

## Configuration

MindMap reads a configuration file from `~/.config/mindmap/config.yaml`. The default configuration file looks like this:

```yaml
# The directory where your notes are stored
data_dir: ~/mindmap

# The file path for your DB
db_path: ~/.config/mindmap/mindmap.db

# The file where the runtime logs are stored
log_path: ~/.config/mindmap/mindmap.log

# A lock file to prevent multiple instances of MindMap from running
lock_path: ~/.mindmap.lock

# The minimum score for a search result to be considered
min_score: 0.2

# The model
model: AllMiniLmL12V2

# The number of search results to return
topk: 10

# Server configuration
server:
  host: 127.0.0.1
  port: 5001
```

## Models

There are a few models to choose from. The models are:
- AllDistilrobertaV1
- AllMiniLmL12V2
- AllMiniLmL6V2
- BertBaseNliMeanTokens
- DistiluseBaseMultilingualCased
- ParaphraseAlbertSmallV2
- SentenceT5Base

## Recommended additional tools

[mindmap.nvim](https://github.com/danimelchor/mindmap.nvim) is a Neovim plugin that allows you to quickly search and edit your MindMap notes.
