use anyhow::{Context, Result};
use bsky_graph::FollowsWriter;
use bsky_graph::{AtProtoGetFollows, DidFileReader};
use clap::Parser;
use std::env;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // Input file containing list of bluesky DID
    #[arg(short, long)]
    input_file: String,
    // page size when doing bluesky request
    #[arg(short, long, default_value_t = 100)]
    limit: u8,
    // buffer size use to write of the parquet file
    #[arg(short, long, default_value_t = 100)]
    buf_size: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let login = env::var("BSKY_LOGIN").context("Missing BSKY_LOGIN environment variable")?;
    let password =
        env::var("BSKY_PASSWORD").context("Missing BSKY_PASSWORD environment variable")?;
    let args = Args::parse();
    let reader = DidFileReader::new(&args.input_file)?;
    // Initialize the agent
    // example login="vincentgauthier.bsky.social" password="x3P!mbUcEcY$9H"
    let atproto: AtProtoGetFollows = AtProtoGetFollows::new(&login, &password, args.limit);
    let mut writer = FollowsWriter::new(atproto, reader, args.buf_size);
    writer.write_follows().await?;
    Ok(())
}
