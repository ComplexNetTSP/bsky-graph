use anyhow::Result;
use atrium_api::types::string::AtIdentifier;
use serde::{Deserialize, Serialize};
use std::future::Future;

// impl future is the way to define async Function Traits in Rust
pub trait AtProtoClient<T>
where
    T: Serialize + Deserialize<'static>,
{
    fn get_graph_w_retry(
        &mut self,
        did: AtIdentifier,
        retries: u32,
    ) -> impl Future<Output = Result<Vec<T>>>;
}
