mod api;
pub mod download;
mod header;
pub mod jeuinfo;
pub mod system;
pub mod userinfo;

use crate::jeuinfo::JeuInfo;
use crate::system::System;
use crate::userinfo::UserInfo;
#[cfg(test)]
use dotenv::dotenv;
use snafu::{ResultExt, Snafu};

#[derive(Debug)]
pub struct ScreenScraper {
  pub user_login: String,
  pub dev_login: String,
  pub soft_name: String,
  /// User info loaded at construction time. Always populated after `new()` succeeds.
  pub user_info: UserInfo,
  user_password: String,
  dev_password: String,
  client: reqwest::blocking::Client,
}

#[derive(Debug, Snafu)]
pub enum Error {
  #[snafu(display("Failed to fetch UserInfo for {}: {}", user, source))]
  UserInfoFailed { user: String, source: api::Error },

  #[snafu(display("Failed to fetch JeuInfo for {}: {}", filename, source))]
  JeuInfoFailed {
    filename: String,
    source: api::Error,
  },

  #[snafu(display("Failed to fetch systems list: {}", source))]
  SystemsListeFailed { source: api::Error },

  #[snafu(display("Failed to search for game '{}': {}", name, source))]
  JeuRechercheFailed { name: String, source: api::Error },
}

type Result<T, E = Error> = std::result::Result<T, E>;

impl ScreenScraper {
  pub fn new(
    user_login: &str,
    user_password: &str,
    dev_login: &str,
    dev_password: &str,
  ) -> Result<ScreenScraper> {
    let client = reqwest::blocking::Client::new();
    let soft_name = "ScreenScraper Rust Library".to_string();
    let query = vec![
      ("devid", dev_login.to_string()),
      ("devpassword", dev_password.to_string()),
      ("softname", soft_name.clone()),
      ("ssid", user_login.to_string()),
      ("sspassword", user_password.to_string()),
      ("output", "json".to_string()),
    ];
    let user_info = api::fetch_user_info(&client, &query).context(UserInfoFailedSnafu {
      user: user_login.to_string(),
    })?;
    Ok(ScreenScraper {
      user_login: user_login.to_string(),
      user_password: user_password.to_string(),
      dev_login: dev_login.to_string(),
      dev_password: dev_password.to_string(),
      soft_name,
      user_info,
      client,
    })
  }

  pub fn jeuinfo(
    &self,
    id: u32,
    filename: &str,
    filesize: u64,
    crc: Option<String>,
    md5: Option<String>,
    sha1: Option<String>,
  ) -> Result<JeuInfo> {
    let mut query = self.base_query();
    query.push(("systemeid", format!("{}", id)));
    query.push(("romnom", filename.to_string()));
    query.push(("romtaille", format!("{}", filesize)));

    if let Some(x) = crc {
      query.push(("crc", x));
    }
    if let Some(x) = md5 {
      query.push(("md5", x));
    }
    if let Some(x) = sha1 {
      query.push(("sha1", x));
    }

    api::fetch_jeu_info(&self.client, &query).context(JeuInfoFailedSnafu {
      filename: filename.to_string(),
    })
  }

  /// Returns the full list of systems known to ScreenScraper.
  pub fn systems_liste(&self) -> Result<Vec<System>> {
    let query = self.base_query();
    api::fetch_systems_liste(&self.client, &query).context(SystemsListeFailedSnafu)
  }

  /// Searches for games by name (up to 30 results, ranked by probability).
  /// `system_id` narrows the search to a specific system when provided.
  pub fn jeu_recherche(&self, system_id: Option<u32>, name: &str) -> Result<Vec<JeuInfo>> {
    let mut query = self.base_query();
    if let Some(id) = system_id {
      query.push(("systemeid", id.to_string()));
    }
    query.push(("recherche", name.to_string()));
    api::fetch_jeu_recherche(&self.client, &query).context(JeuRechercheFailedSnafu {
      name: name.to_string(),
    })
  }

  /// Fetches game info using a known ScreenScraper game ID.
  ///
  /// Unlike [`jeuinfo`], this does not require ROM hashes — the game is
  /// identified directly. ROM-specific fields (`rom`, `roms`) will be absent.
  pub fn jeuinfo_by_gameid(&self, system_id: u32, game_id: u32) -> Result<JeuInfo> {
    let mut query = self.base_query();
    query.push(("systemeid", system_id.to_string()));
    query.push(("gameid", game_id.to_string()));
    query.push(("romnom", String::new()));
    api::fetch_jeu_info(&self.client, &query).context(JeuInfoFailedSnafu {
      filename: format!("gameid:{}", game_id),
    })
  }

  pub fn media_download<'a>(&'a self, media: &'a jeuinfo::Media) -> download::MediaDownload<'a> {
    download::MediaDownload::new(&self.client, media)
  }

  fn base_query(&self) -> Vec<(&str, String)> {
    vec![
      ("devid", self.dev_login.clone()),
      ("devpassword", self.dev_password.clone()),
      ("softname", self.soft_name.clone()),
      ("ssid", self.user_login.clone()),
      ("sspassword", self.user_password.clone()),
      ("output", "json".to_string()),
    ]
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  #[ignore = "requires valid API credentials in .env"]
  fn it_works() {
    dotenv().ok();

    let ss = ScreenScraper::new(
      &std::env::var("SS_USER_LOGIN").unwrap(),
      &std::env::var("SS_USER_PASSWORD").unwrap(),
      &std::env::var("SS_DEV_LOGIN").unwrap(),
      &std::env::var("SS_DEV_PASSWORD").unwrap(),
    )
    .unwrap();
    println!("{:#?}", ss);

    let ji = ss
      .jeuinfo(
        1,
        "Sonic The Hedgehog (World).zip",
        749652,
        None,
        None,
        None,
      )
      .unwrap();
    println!("{:#?}", ji);

    let systems = ss.systems_liste().unwrap();
    println!("Systems count: {}", systems.len());

    let results = ss.jeu_recherche(Some(1), "Sonic").unwrap();
    println!("Search results: {}", results.len());
  }
}
