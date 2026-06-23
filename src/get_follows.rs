use crate::{GetGraphError, atproto_client::AtProtoClient, follows::Follows};
use atrium_api::{
    agent::atp_agent::{AtpAgent, store::MemorySessionStore},
    app::bsky::actor::defs::ProfileView,
    app::bsky::graph::get_follows,
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
pub struct AtProtoGetFollows {
    login_name: String,
    password: String,
    agent: AtpAgent<MemorySessionStore, ReqwestClient>,
    is_login: bool,
    limit: Option<LimitedNonZeroU8<100>>,
    rate_limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
}

impl AtProtoGetFollows {
    pub fn new(login: &str, password: &str, limit: u8) -> Self {
        let limit = LimitedNonZeroU8::<100>::try_from(limit).ok();
        // Initialize the agent
        let agent = AtpAgent::new(
            ReqwestClient::new("https://bsky.social"),
            MemorySessionStore::default(),
        );
        // hard coded default Bluesky query limit
        let rate_limiter = RateLimiter::direct(Quota::per_second(NonZeroU32::new(5).unwrap()));
        AtProtoGetFollows {
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

    pub async fn get_follows(
        &mut self,
        did: &AtIdentifier,
    ) -> Result<(ProfileView, Vec<ProfileView>), GetGraphError> {
        if !self.is_login {
            match self.login().await {
                Ok(_) => info!("Get_follow Login sucessful to the bluesky api"),
                Err(e) => {
                    error!("Get_follow unable to login to the bluesky api: {}", e);
                    return Err(GetGraphError::UnableToLogin);
                }
            }
        }
        let mut cursor = None;
        let subject;
        let mut all_follows = Vec::new();
        // Call the getFollowers endpoint
        loop {
            // Apply rate limit
            self.rate_limiter.until_ready().await;
            let message = self
                .agent
                .api
                .app
                .bsky
                .graph
                .get_follows(
                    get_follows::ParametersData {
                        actor: did.clone(),
                        cursor: cursor.clone(),
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

            all_follows.extend(response.data.follows);
            // continue if other data in the response
            if let Some(next_cursor) = response.data.cursor {
                cursor = Some(next_cursor);
            } else {
                subject = response.data.subject;
                break;
            }
        }
        Ok((subject, all_follows))
    }

    pub async fn get_follows_w_retry(
        &mut self,
        did: AtIdentifier,
        max_retry: u32,
    ) -> anyhow::Result<(ProfileView, Vec<ProfileView>)> {
        for retry in 0..max_retry {
            match self.get_follows(&did).await {
                Ok((subject, follows)) => return Ok((subject, follows)),
                Err(GetGraphError::RateLimited) => {
                    error!("Retry {}/{} after get rate mlimited", retry + 1, max_retry);
                    sleep(Duration::from_secs(2u64.pow(retry))).await;
                }
                Err(_) => break,
            }
        }
        error!("Unable to fetch Follows for {:?}", did);
        Err(anyhow::anyhow!("Unable to fetch Follows for {:?}", did))
    }
}

impl AtProtoClient<Follows> for AtProtoGetFollows {
    async fn get_graph_w_retry(
        &mut self,
        did: AtIdentifier,
        retries: u32,
    ) -> anyhow::Result<Vec<Follows>> {
        let (subject, follows) = self.get_follows_w_retry(did, retries).await?;
        Follows::create_edge_list(subject, follows)
    }

    fn type_name() -> String {
        "follows".to_string()
    }
}
