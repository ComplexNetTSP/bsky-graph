use crate::{atproto_client::AtProtoClient, follows::Follows};
use anyhow::{Context, Result};
use atrium_api::{
    agent::atp_agent::{AtpAgent, store::MemorySessionStore},
    app::bsky::actor::defs::ProfileView,
    app::bsky::graph::get_follows,
    types::{LimitedNonZeroU8, string::AtIdentifier},
};
use atrium_xrpc_client::reqwest::ReqwestClient;
use tokio::time::{Duration, sleep};

#[allow(dead_code)]
pub struct AtProtoGetFollows {
    login_name: String,
    password: String,
    agent: AtpAgent<MemorySessionStore, ReqwestClient>,
    is_login: bool,
    limit: Option<LimitedNonZeroU8<100>>,
}

impl AtProtoGetFollows {
    pub fn new(login: &str, password: &str, limit: u8) -> Self {
        let limit = LimitedNonZeroU8::<100>::try_from(limit).ok();
        // Initialize the agent
        let agent = AtpAgent::new(
            ReqwestClient::new("https://bsky.social"),
            MemorySessionStore::default(),
        );
        AtProtoGetFollows {
            login_name: login.to_string(),
            password: password.to_string(),
            agent,
            is_login: false,
            limit,
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
    ) -> anyhow::Result<(ProfileView, Vec<ProfileView>)> {
        if !self.is_login {
            match self.login().await {
                Ok(_) => eprintln!("Login sucessful"),
                Err(e) => {
                    return Err(anyhow::anyhow!("Unable to login:{}", e));
                }
            }
        }
        let mut cursor = None;
        let subject;
        let mut all_follows = Vec::new();
        // Call the getFollowers endpoint
        loop {
            let response = self
                .agent
                .api
                .app
                .bsky
                .graph
                .get_follows(
                    get_follows::ParametersData {
                        actor: did.clone(),
                        cursor,
                        limit: self.limit,
                    }
                    .into(),
                )
                .await
                .context("Failled to get followers")?;
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
                Err(e) => {
                    eprintln!(
                        "Retry {}/{} - Failed to fetch follows: {}",
                        retry + 1,
                        max_retry,
                        e
                    );
                    sleep(Duration::from_secs(2u64.pow(retry))).await;
                }
            }
        }
        Err(anyhow::anyhow!(
            "Unable to fetch Follows for {:?} after {} retries.",
            did,
            max_retry
        ))
    }
}

impl AtProtoClient<Follows> for AtProtoGetFollows {
    async fn get_graph_w_retry(&mut self, did: AtIdentifier, retries: u32) -> Result<Vec<Follows>> {
        let (subject, follows) = self.get_follows_w_retry(did, retries).await?;
        Follows::create_edge_list(subject, follows)
    }

    fn type_name() -> String {
        "follows".to_string()
    }
}
