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

/// Why an API call failed, as far as the library can tell.
///
/// ScreenScraper reports its errors through the HTTP status code (see the table
/// in the API documentation), so most variants map to one status. The last three
/// cover failures that never reached that point, or that came back malformed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiFailure {
  /// 404 — game or ROM not found. The only failure that means "ScreenScraper
  /// genuinely does not know this ROM".
  NotFound,
  /// 400 — malformed request: missing mandatory field, ROM name containing a
  /// path, badly formatted crc/md5/sha1. A caller bug.
  BadRequest,
  /// 401 — API closed to non-members or inactive members; server saturated
  /// (CPU > 60 %).
  ServerBusy,
  /// 403 — wrong developer credentials.
  BadDevCredentials,
  /// 423 — API fully closed, serious server problems.
  ApiClosed,
  /// 426 — scraping software blacklisted (non-compliant or obsolete version).
  Blacklisted,
  /// 429 — thread limit reached (per member, per minute, leechers, or global).
  ThreadLimit,
  /// 430 — daily scrape quota exceeded.
  QuotaExceeded,
  /// 431 — too many unrecognised ROMs today.
  KoQuotaExceeded,
  /// Any other non-2xx status.
  Http(u16),
  /// The request never completed: timeout, DNS failure, connection refused.
  Transport,
  /// A 2xx response the library could not parse.
  Malformed,
  /// `header.success == "false"` in an otherwise valid response.
  Api,
}

impl ApiFailure {
  /// `true` when retrying the same call has a chance of succeeding.
  ///
  /// Only [`ApiFailure::ThreadLimit`], [`ApiFailure::ServerBusy`] and
  /// [`ApiFailure::Transport`] qualify. Quota failures (430, 431) and a closed
  /// API (423) hold for the rest of the run — retrying them only burns error
  /// quota. The library never retries nor sleeps on its own; that policy
  /// belongs to the caller.
  pub fn is_retryable(&self) -> bool {
    matches!(
      self,
      ApiFailure::ThreadLimit | ApiFailure::ServerBusy | ApiFailure::Transport
    )
  }
}

/// Maps a ScreenScraper HTTP status to its meaning. Pure, and the single place
/// where the status table lives.
fn classify(status: u16) -> ApiFailure {
  match status {
    400 => ApiFailure::BadRequest,
    401 => ApiFailure::ServerBusy,
    403 => ApiFailure::BadDevCredentials,
    404 => ApiFailure::NotFound,
    423 => ApiFailure::ApiClosed,
    426 => ApiFailure::Blacklisted,
    429 => ApiFailure::ThreadLimit,
    430 => ApiFailure::QuotaExceeded,
    431 => ApiFailure::KoQuotaExceeded,
    other => ApiFailure::Http(other),
  }
}

impl Error {
  /// Classifies the failure behind this error.
  pub fn failure(&self) -> ApiFailure {
    let source = match self {
      Error::UserInfoFailed { source, .. }
      | Error::JeuInfoFailed { source, .. }
      | Error::SystemsListeFailed { source }
      | Error::JeuRechercheFailed { source, .. } => source,
    };
    match source {
      api::Error::Http { status, .. } => classify(*status),
      api::Error::Request { .. } => ApiFailure::Transport,
      api::Error::Parse { .. } => ApiFailure::Malformed,
      api::Error::Api { .. } => ApiFailure::Api,
    }
  }

  /// `true` when retrying the same call has a chance of succeeding.
  /// See [`ApiFailure::is_retryable`].
  pub fn is_retryable(&self) -> bool {
    self.failure().is_retryable()
  }

  /// Shorthand for `failure() == ApiFailure::NotFound`.
  pub fn is_not_found(&self) -> bool {
    self.failure() == ApiFailure::NotFound
  }
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

  fn http(status: u16) -> api::Error {
    api::Error::Http {
      url: "https://www.screenscraper.fr/api2/jeuInfos.php".to_string(),
      status,
      body: String::new(),
    }
  }

  // --- classify ---

  #[test]
  fn classify_maps_every_documented_status() {
    assert_eq!(classify(400), ApiFailure::BadRequest);
    assert_eq!(classify(401), ApiFailure::ServerBusy);
    assert_eq!(classify(403), ApiFailure::BadDevCredentials);
    assert_eq!(classify(404), ApiFailure::NotFound);
    assert_eq!(classify(423), ApiFailure::ApiClosed);
    assert_eq!(classify(426), ApiFailure::Blacklisted);
    assert_eq!(classify(429), ApiFailure::ThreadLimit);
    assert_eq!(classify(430), ApiFailure::QuotaExceeded);
    assert_eq!(classify(431), ApiFailure::KoQuotaExceeded);
  }

  #[test]
  fn classify_falls_back_to_http_for_unknown_status() {
    assert_eq!(classify(500), ApiFailure::Http(500));
    assert_eq!(classify(402), ApiFailure::Http(402));
  }

  // --- is_retryable ---

  #[test]
  fn is_retryable_only_for_transient_failures() {
    assert!(ApiFailure::ThreadLimit.is_retryable());
    assert!(ApiFailure::ServerBusy.is_retryable());
    assert!(ApiFailure::Transport.is_retryable());

    assert!(!ApiFailure::NotFound.is_retryable());
    assert!(!ApiFailure::BadRequest.is_retryable());
    assert!(!ApiFailure::BadDevCredentials.is_retryable());
    assert!(!ApiFailure::ApiClosed.is_retryable());
    assert!(!ApiFailure::Blacklisted.is_retryable());
    assert!(!ApiFailure::QuotaExceeded.is_retryable());
    assert!(!ApiFailure::KoQuotaExceeded.is_retryable());
    assert!(!ApiFailure::Http(500).is_retryable());
    assert!(!ApiFailure::Malformed.is_retryable());
    assert!(!ApiFailure::Api.is_retryable());
  }

  // --- failure() traversal, one case per wrapper ---

  #[test]
  fn failure_traverses_user_info_failed() {
    let err = Error::UserInfoFailed {
      user: "someone".to_string(),
      source: http(403),
    };
    assert_eq!(err.failure(), ApiFailure::BadDevCredentials);
    assert!(!err.is_retryable());
  }

  #[test]
  fn failure_traverses_jeu_info_failed() {
    let err = Error::JeuInfoFailed {
      filename: "Sonic.zip".to_string(),
      source: http(404),
    };
    assert_eq!(err.failure(), ApiFailure::NotFound);
    assert!(err.is_not_found());
    assert!(!err.is_retryable());
  }

  #[test]
  fn failure_traverses_systems_liste_failed() {
    let err = Error::SystemsListeFailed { source: http(429) };
    assert_eq!(err.failure(), ApiFailure::ThreadLimit);
    assert!(err.is_retryable());
    assert!(!err.is_not_found());
  }

  #[test]
  fn failure_traverses_jeu_recherche_failed() {
    let err = Error::JeuRechercheFailed {
      name: "Sonic".to_string(),
      source: http(430),
    };
    assert_eq!(err.failure(), ApiFailure::QuotaExceeded);
    assert!(!err.is_retryable());
  }

  // --- failure() on non-HTTP sources ---

  #[test]
  fn failure_reports_malformed_for_parse_errors() {
    let source = api::Error::Parse {
      source: serde_json::from_str::<u32>("nope").unwrap_err(),
    };
    let err = Error::SystemsListeFailed { source };
    assert_eq!(err.failure(), ApiFailure::Malformed);
  }

  #[test]
  fn failure_reports_api_for_header_errors() {
    let source = api::Error::Api {
      message: "Erreur : Acces WS non autorise !".to_string(),
    };
    let err = Error::SystemsListeFailed { source };
    assert_eq!(err.failure(), ApiFailure::Api);
  }

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
