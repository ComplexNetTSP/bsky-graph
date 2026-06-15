use anyhow::Context;
use atrium_api::{
    agent::atp_agent::{AtpAgent, store::MemorySessionStore},
    app::bsky::actor::defs::ProfileView,
    app::bsky::graph::get_follows,
    types::{LimitedNonZeroU8, string::AtIdentifier},
};
use atrium_xrpc_client::reqwest::ReqwestClient;

#[allow(dead_code)]
pub struct AtProtoGetFollows {
    login_name: String,
    password: String,
    agent: AtpAgent<MemorySessionStore, ReqwestClient>,
    is_login: bool,
}

impl AtProtoGetFollows {
    pub fn new(login: &str, password: &str) -> Self {
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
        did: AtIdentifier,
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
        let limit = LimitedNonZeroU8::try_from(10).map_err(anyhow::Error::msg)?;
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
                        cursor: cursor,
                        limit: Some(limit),
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
}
