use anyhow::Result;
use atrium_api::app::bsky::actor::defs::ProfileView;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Follows {
    // subject
    pub did: String,
    pub handle: String,
    pub avatar: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    // follower
    pub follows_did: String,
    pub follows_handle: String,
    pub follows_avatar: Option<String>,
    pub follows_display_name: Option<String>,
    pub follows_description: Option<String>,
}

impl Follows {
    pub fn create_edge_list(
        suject: ProfileView,
        follows: Vec<ProfileView>,
    ) -> Result<Vec<Follows>> {
        let list = follows
            .iter()
            .map(|item| Follows {
                did: suject.did.to_string(),
                handle: suject.handle.to_string(),
                avatar: suject.avatar.clone(),
                display_name: suject.display_name.clone(),
                description: suject.description.clone(),
                follows_did: item.did.to_string(),
                follows_handle: item.handle.to_string(),
                follows_avatar: item.avatar.clone(),
                follows_display_name: item.display_name.clone(),
                follows_description: item.description.clone(),
            })
            .collect();
        Ok(list)
    }
}
