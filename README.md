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
- Rate limiting to prevent API ban
- Typed error handling with smart retry logic (retries only on rate limit errors)

## Usage

```bash
# Set up environment variables
cp .env.example .env
# Edit .env with your BSKY_LOGIN and BSKY_PASSWORD

# Fetch follows (default)
cargo run -- --input-file users.txt --output-dir ./output

# Fetch followers instead
cargo run -- --input-file users.txt --output-dir ./output --follower
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
| `--follower` | `-f` | false | Fetch followers instead of follows |

## Requirements

- Rust 2024 edition
- Bluesky account credentials

## Changelog

### v0.1.6 (2026-06-23)
- Added `--follower` flag to choose between fetching followers or follows
- Enhanced command-line interface with better descriptions
- Fixed rate limiter: unified to 5 requests/second for both followers and follows
- Added info logging to indicate which type is being fetched

### v0.1.5 (2026-06-23)
- Added `thiserror` dependency for typed error handling
- Created `GetGraphError` enum with variants for rate limiting, bad requests, login failures, and unexpected errors
- Improved error handling in `get_follows` and `get_follower` with smart retry logic (only retry on rate limit errors)
- Adjusted rate limits: 5 requests/second for follows, 3 requests/second for followers

### v0.1.3 (2026-06-23)
- Changed rate limit from 600 to 3 requests/second

### v0.1.2 (2026-06-23)
- Added rate limiting using `governor` crate (600 requests/second) to prevent API bans
- Removed manual 100ms delay between requests
- Improved error handling in parquet writer

### v0.1.0 (2026-06-15)
- Initial release

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

Vincent Gauthier <vincent.gauthier@telecom-sudparis.eu> - Telecom SudParis
