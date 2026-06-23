use crate::{GetGraphError, atproto_client::AtProtoClient, follower::Follower};
use anyhow::{Context, Result};
use atrium_api::{
    agent::atp_agent::{AtpAgent, store::MemorySessionStore},
    app::bsky::actor::defs::ProfileView,
    app::bsky::graph::get_followers,
    types::{LimitedNonZeroU8, string::AtIdentifier},
    xrpc::Error as XrpcError,
};
use atrium_xrpc_client::reqwest::ReqwestClient;
use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use log::{error, info};
use std::num::NonZeroU32;
use tokio::time::{Duration, sleep};

#[allow(dead_code)]
pub struct AtProtoGetFollower {
    login_name: String,
    password: String,
    agent: AtpAgent<MemorySessionStore, ReqwestClient>,
    is_login: bool,
    limit: Option<LimitedNonZeroU8<100>>,
    rate_limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
}

impl AtProtoGetFollower {
    pub fn new(login: &str, password: &str, limit: u8) -> Self {
        let limit = LimitedNonZeroU8::<100>::try_from(limit).ok();
        // Initialize the agent
        let agent = AtpAgent::new(
            ReqwestClient::new("https://bsky.social"),
            MemorySessionStore::default(),
        );
        // hard coded default Bluesky query limit
        let rate_limiter = RateLimiter::direct(Quota::per_second(NonZeroU32::new(3).unwrap()));
        AtProtoGetFollower {
            login_name: login.to_string(),
            password: password.to_string(),
            agent,
            is_login: false,
            limit,
            rate_limiter,
        }
    }

    async fn login(&mut self) -> anyhow::Result<()> {
        // Log in (replace with your credentials)
        self.agent.login(&self.login_name, &self.password).await?;
        self.is_login = true;
        Ok(())
    }

    pub async fn get_follower(
        &mut self,
        did: &AtIdentifier,
    ) -> Result<(ProfileView, Vec<ProfileView>), GetGraphError> {
        if !self.is_login {
            match self.login().await {
                Ok(_) => info!("Get_Follower login sucessful to bluesky api"),
                Err(e) => {
                    error!("Get_Follower unable to login to bluesky api: {}", e);
                    return Err(GetGraphError::UnableToLogin);
                }
            }
        }
        let mut cursor = None;
        let subject;
        let mut all_followers = Vec::new();
        // Call the getFollowers endpoint
        loop {
            // Apply rate limit
            self.rate_limiter.until_ready().await;
            // send query
            let message = self
                .agent
                .api
                .app
                .bsky
                .graph
                .get_followers(
                    get_followers::ParametersData {
                        actor: did.clone(),
                        cursor,
                        limit: self.limit,
                    }
                    .into(),
                )
                .await;

            let response = match message {
                Ok(response) => response,
                Err(XrpcError::XrpcResponse(response)) => match response.status.as_u16() {
                    400 => return Err(GetGraphError::BadRequest),
                    429 => return Err(GetGraphError::RateLimited),
                    _ => return Err(GetGraphError::UnexpectedResponseType),
                },
                Err(_) => return Err(GetGraphError::UnexpectedResponseType),
            };

            all_followers.extend(response.data.followers);
            if let Some(next_cursor) = response.data.cursor {
                cursor = Some(next_cursor);
            } else {
                subject = response.data.subject;
                break;
            }
        }
        Ok((subject, all_followers))
    }

    pub async fn get_follower_w_retry(
        &mut self,
        did: AtIdentifier,
        max_retry: u32,
    ) -> anyhow::Result<(ProfileView, Vec<ProfileView>)> {
        for retry in 0..max_retry {
            match self.get_follower(&did).await {
                Ok((subject, follows)) => return Ok((subject, follows)),
                Err(GetGraphError::RateLimited) => {
                    error!(
                        "GetFollower Retry {}/{} after get rate mlimited",
                        retry + 1,
                        max_retry
                    );
                    sleep(Duration::from_secs(2u64.pow(retry))).await;
                }
                Err(_) => break,
            }
        }
        error!("Unable to fetch Follower for {:?}.", did);
        Err(anyhow::anyhow!("Unable to fetch Follower for {:?}", did))
    }
}

impl AtProtoClient<Follower> for AtProtoGetFollower {
    async fn get_graph_w_retry(
        &mut self,
        did: AtIdentifier,
        retries: u32,
    ) -> Result<Vec<Follower>> {
        let (subject, follower) = self
            .get_follower_w_retry(did, retries)
            .await
            .context("Error fetching follower")?;
        Ok(Follower::create_edge_list(subject, follower))
    }

    fn type_name() -> String {
        "follower".to_string()
    }
}
