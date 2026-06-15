use anyhow::{Context, Result};
use atrium_api::types::string::AtIdentifier;
use atrium_api::types::string::Handle;
use bsky_graph::{AtProtoGetFollows, Follows};
use clap::Parser;
use std::env;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // Input file containing list of bluesky DID
    #[arg(short, long)]
    input_file: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let login = env::var("BSKY_LOGIN").context("Missing BSKY_LOGIN environment variable")?;
    let password =
        env::var("BSKY_PASSWORD").context("Missing BSKY_PASSWORD environment variable")?;
    let args = Args::parse();
    // Initialize the agent
    // example login="vincentgauthier.bsky.social" password="x3P!mbUcEcY$9H"
    let mut atproto = AtProtoGetFollows::new(&login, &password);
    // Create from a handle string
    let handle =
        Handle::new("vincentgauthier.bsky.social".to_string()).expect("failed to parse handle");
    let at_id = AtIdentifier::Handle(handle);
    // Print the followers
    let (subject, follows) = atproto.get_follows(at_id).await?;
    let subject_follows_edge = Follows::create_edge_list(subject, follows)?;
    println!("{:?}", subject_follows_edge);
    Ok(())
}
