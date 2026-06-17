use atrium_api::app::bsky::actor::defs::ProfileView;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct Follower {
    // subject
    pub did: String,
    pub handle: String,
    pub avatar: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    // follower
    pub follower_did: String,
    pub follower_handle: String,
    pub follower_avatar: Option<String>,
    pub follower_display_name: Option<String>,
    pub follower_description: Option<String>,
}

impl Follower {
    pub fn create_edge_list(suject: ProfileView, followers: Vec<ProfileView>) -> Vec<Follower> {
        followers
            .iter()
            .map(|item| Follower {
                did: suject.did.to_string(),
                handle: suject.handle.to_string(),
                avatar: suject.avatar.clone(),
                display_name: suject.display_name.clone(),
                description: suject.description.clone(),
                follower_did: item.did.to_string(),
                follower_handle: item.handle.to_string(),
                follower_avatar: item.avatar.clone(),
                follower_display_name: item.display_name.clone(),
                follower_description: item.description.clone(),
            })
            .collect()
    }
}
