# bsky-graph
[![GitHub Release](https://img.shields.io/github/v/release/ComplexNetTSP/bsky-graph)](https://github.com/ComplexNetTSP/bsky-graph/releases)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/ComplexNetTSP/bsky-graph/build.yaml)](https://github.com/ComplexNetTSP/bsky-graph/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Dump followers and follows DIDs for Bluesky users to Parquet files.

## Description

A command-line tool that retrieves follower and following relationships from Bluesky (AT Protocol) and exports them to Apache Parquet files for analysis.

## Features

- Retrieve follows (users that a given account follows)
- Retrieve followers (users that follow a given account)
- Output to Parquet format for efficient storage and querying
- Configurable page size, buffer size, and retry logic
- Progress tracking with progress bars

## Usage

```bash
# Set up environment variables
cp .env.example .env
# Edit .env with your BSKY_LOGIN and BSKY_PASSWORD

# Run the tool
cargo run -- --input-file users.txt --output-dir ./output
```

## Command Line Arguments

| Argument | Short | Default | Description |
|----------|-------|---------|-------------|
| `--input-file` | `-i` | (required) | Input file containing list of Bluesky DIDs |
| `--limit` | `-l` | 100 | Page size for Bluesky API requests |
| `--buf-size` | `-b` | 100 | Buffer size for Parquet writing |
| `--output-dir` | `-o` | `./output` | Output directory for Parquet files |
| `--log-file` | `-w` | `bsky-graph.log` | Log file path |
| `--max-retry` | `-m` | 10 | Maximum number of retries before failing |

## Requirements

- Rust 2024 edition
- Bluesky account credentials

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

Vincent Gauthier <vincent.gauthier@telecom-sudparis.eu> - Telecom SudParis
