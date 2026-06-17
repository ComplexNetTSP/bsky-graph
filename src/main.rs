use anyhow::{Context, Result};
use bsky_graph::{
    AtProtoGetFollower, AtProtoGetFollows, DidFileReader, ParquetWriter, utils::setup_logger,
};
use clap::Parser;
use std::env;

#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = "Dump followers and follows DIDs for Bluesky users to Parquet files"
)]
struct Args {
    /// Input file containing list of bluesky DID
    #[arg(short, long)]
    input_file: String,
    /// Page size when doing bluesky request
    #[arg(short, long, default_value_t = 100)]
    limit: u8,
    /// Buffer size use to write of the parquet file
    #[arg(short, long, default_value_t = 100)]
    buf_size: usize,
    /// Output directory of the parquet files
    #[arg(short, long, default_value_t = String::from("./output"))]
    output_dir: String,
    /// Log file
    #[arg(short='w', long, default_value_t = String::from("bsky-graph.log"))]
    log_file: String,
    /// maximum of request retry before failling
    #[arg(short, long, default_value_t = 10)]
    max_retry: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file (silently ignores if missing)
    dotenv::dotenv().ok();
    let login = env::var("BSKY_LOGIN").context("Missing BSKY_LOGIN environment variable")?;
    let password =
        env::var("BSKY_PASSWORD").context("Missing BSKY_PASSWORD environment variable")?;
    let args = Args::parse();
    setup_logger(&args.log_file)
        .context(format!("unable to create log file {}", &args.log_file))?;

    // Initialize the agent
    let atproto_follows: AtProtoGetFollows = AtProtoGetFollows::new(&login, &password, args.limit);
    let atproto_follower: AtProtoGetFollower =
        AtProtoGetFollower::new(&login, &password, args.limit);

    let mut follows_writer = ParquetWriter::new(
        atproto_follows,
        DidFileReader::new(&args.input_file)?,
        args.buf_size,
        &args.output_dir,
        args.max_retry,
    );
    let mut follower_writer = ParquetWriter::new(
        atproto_follower,
        DidFileReader::new(&args.input_file)?,
        args.buf_size,
        &args.output_dir,
        args.max_retry,
    );
    follows_writer.write().await?;
    follower_writer.write().await?;
    Ok(())
}
