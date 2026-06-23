pub mod atproto_client;
pub mod follower;
pub mod follows;
pub mod get_follower;
pub mod get_follows;
pub mod parquet_writer;
pub mod read_did;
pub mod utils;
// rexport
pub use follower::Follower;
pub use follows::Follows;
pub use get_follower::AtProtoGetFollower;
pub use get_follows::AtProtoGetFollows;
pub use parquet_writer::ParquetWriter;
pub use read_did::DidFileReader;

/// An enum of possible error kinds.
#[derive(thiserror::Error, Debug)]
pub enum GetGraphError {
    #[error("rate limited")]
    RateLimited,
    #[error("bad request")]
    BadRequest,
    #[error("unable to login")]
    UnableToLogin,
    #[error("unexepected error")]
    UnexpectedResponseType,
}
