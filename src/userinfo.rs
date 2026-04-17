use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfo {
    pub id: String,
    pub numid: Option<String>,
    pub niveau: String,
    pub contribution: String,
    pub uploadsysteme: String,
    pub uploadinfos: String,
    pub romasso: String,
    pub uploadmedia: String,
    pub propositionok: Option<String>,
    pub propositionko: Option<String>,
    pub quotarefu: Option<String>,
    pub maxthreads: String,
    pub maxdownloadspeed: String,
    pub requeststoday: Option<String>,
    pub requestskotoday: Option<String>,
    pub maxrequestspermin: Option<String>,
    pub maxrequestsperday: Option<String>,
    pub maxrequestskoperday: Option<String>,
    pub visites: String,
    pub datedernierevisite: String,
    pub favregion: String,
}
