use super::header::Header;
use super::jeuinfo::JeuInfo;
use super::system::System;
use super::userinfo::UserInfo;
use serde::Deserialize;
use snafu::{ensure, ResultExt, Snafu};

#[derive(Deserialize)]
struct ResponseUserInfo {
  ssuser: UserInfo,
}

#[derive(Deserialize)]
struct UserInfosResult {
  header: Header,
  response: Option<ResponseUserInfo>,
}

#[derive(Deserialize)]
struct ResponseJeuInfo {
  jeu: JeuInfo,
}

#[derive(Deserialize)]
struct JeuInfosResult {
  header: Header,
  response: Option<ResponseJeuInfo>,
}

#[derive(Deserialize)]
struct ResponseSystemesListe {
  systemes: Vec<System>,
}

#[derive(Deserialize)]
struct SystemesListeResult {
  header: Header,
  response: Option<ResponseSystemesListe>,
}

#[derive(Deserialize)]
struct ResponseJeuRecherche {
  /// ScreenScraper says "no result" with a list holding a *single empty object*
  /// (HTTP 200, `success: "true"`), not with an empty list nor with a 404.
  /// Deserializing straight into `Vec<JeuInfo>` therefore fails the whole call
  /// on `missing field id`, and the caller sees `Malformed` — indistinguishable
  /// from a genuinely corrupt response.
  #[serde(deserialize_with = "jeux_without_empty_entries")]
  jeux: Vec<JeuInfo>,
}

/// Drops empty objects from a `jeux` list, and nothing else: a non-empty entry
/// that `JeuInfo` refuses is a real schema divergence and must keep failing the
/// call, rather than silently vanishing from a result list.
fn jeux_without_empty_entries<'de, D>(deserializer: D) -> Result<Vec<JeuInfo>, D::Error>
where
  D: serde::Deserializer<'de>,
{
  use serde::de::Error as _;
  Vec::<serde_json::Value>::deserialize(deserializer)?
    .into_iter()
    .filter(|v| !matches!(v.as_object(), Some(o) if o.is_empty()))
    .map(|v| serde_json::from_value(v).map_err(D::Error::custom))
    .collect()
}

#[derive(Deserialize)]
struct JeuRechercheResult {
  header: Header,
  response: Option<ResponseJeuRecherche>,
}

#[derive(Debug, Snafu)]
pub enum Error {
  #[snafu(display("Failed to reach API at {}: {}", url, source))]
  Request { url: String, source: reqwest::Error },

  #[snafu(display("API returned HTTP {} for {}: {}", status, url, body))]
  Http {
    url: String,
    status: u16,
    body: String,
  },

  #[snafu(display("Failed to parse API response: {}", source))]
  Parse { source: serde_json::Error },

  #[snafu(display("API error: {}", message))]
  Api { message: String },
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub fn fetch_user_info(
  client: &reqwest::blocking::Client,
  query: &[(&str, String)],
) -> Result<UserInfo> {
  let url = "https://www.screenscraper.fr/api2/ssuserInfos.php";
  parse_user_info(&get(client, url, query)?)
}

pub fn fetch_jeu_info(
  client: &reqwest::blocking::Client,
  query: &[(&str, String)],
) -> Result<JeuInfo> {
  let url = "https://www.screenscraper.fr/api2/jeuInfos.php";
  parse_jeu_info(&get(client, url, query)?)
}

pub fn fetch_systems_liste(
  client: &reqwest::blocking::Client,
  query: &[(&str, String)],
) -> Result<Vec<System>> {
  let url = "https://www.screenscraper.fr/api2/systemesListe.php";
  parse_systems_liste(&get(client, url, query)?)
}

pub fn fetch_jeu_recherche(
  client: &reqwest::blocking::Client,
  query: &[(&str, String)],
) -> Result<Vec<JeuInfo>> {
  let url = "https://www.screenscraper.fr/api2/jeuRecherche.php";
  parse_jeu_recherche(&get(client, url, query)?)
}

fn parse_user_info(body: &str) -> Result<UserInfo> {
  let data: UserInfosResult = serde_json::from_str(body).context(ParseSnafu)?;
  ensure!(
    data.header.success == "true",
    ApiSnafu {
      message: data.header.error.clone()
    }
  );
  data
    .response
    .ok_or_else(|| Error::Api {
      message: "API returned success but no response".to_string(),
    })
    .map(|r| r.ssuser)
}

fn parse_jeu_info(body: &str) -> Result<JeuInfo> {
  let data: JeuInfosResult = serde_json::from_str(body).context(ParseSnafu)?;
  ensure!(
    data.header.success == "true",
    ApiSnafu {
      message: data.header.error.clone()
    }
  );
  data
    .response
    .ok_or_else(|| Error::Api {
      message: "API returned success but no response".to_string(),
    })
    .map(|r| r.jeu)
}

fn parse_systems_liste(body: &str) -> Result<Vec<System>> {
  let data: SystemesListeResult = serde_json::from_str(body).context(ParseSnafu)?;
  ensure!(
    data.header.success == "true",
    ApiSnafu {
      message: data.header.error.clone()
    }
  );
  data
    .response
    .ok_or_else(|| Error::Api {
      message: "API returned success but no response".to_string(),
    })
    .map(|r| r.systemes)
}

fn parse_jeu_recherche(body: &str) -> Result<Vec<JeuInfo>> {
  let data: JeuRechercheResult = serde_json::from_str(body).context(ParseSnafu)?;
  ensure!(
    data.header.success == "true",
    ApiSnafu {
      message: data.header.error.clone()
    }
  );
  data
    .response
    .ok_or_else(|| Error::Api {
      message: "API returned success but no response".to_string(),
    })
    .map(|r| r.jeux)
}

/// Maximum number of characters kept from an error response body.
const BODY_MAX: usize = 200;

fn truncated(body: &str) -> String {
  body.chars().take(BODY_MAX).collect()
}

/// ScreenScraper reports its errors through the HTTP status code, not through
/// `header.success` — an error body is not even JSON.
///
/// `url` is the bare endpoint, never the request URL: the query string carries
/// `devpassword` and `sspassword` and must never reach an error message.
fn get(client: &reqwest::blocking::Client, url: &str, query: &[(&str, String)]) -> Result<String> {
  let response = client
    .get(url)
    .query(query)
    .send()
    .context(RequestSnafu { url })?;
  let status = response.status();
  let body = response.text().context(RequestSnafu { url })?;
  ensure!(
    status.is_success(),
    HttpSnafu {
      url,
      status: status.as_u16(),
      body: truncated(&body),
    }
  );
  Ok(body)
}

#[cfg(test)]
mod tests {
  use super::*;

  const HEADER_OK: &str = r#"{
        "APIversion": "2.0",
        "dateTime": "2026-04-17 22:40:51",
        "commandRequested": "https://api.screenscraper.fr/api2/ssuserInfos.php",
        "success": "true",
        "error": ""
    }"#;

  const HEADER_ERR: &str = r#"{
        "APIversion": "2.0",
        "dateTime": "2026-04-17 22:40:51",
        "commandRequested": "https://api.screenscraper.fr/api2/ssuserInfos.php",
        "success": "false",
        "error": "Erreur : Acces WS non autorise !"
    }"#;

  const SSUSER: &str = r#"{
        "id": "testuser",
        "numid": "9611",
        "niveau": "11",
        "contribution": "0",
        "uploadsysteme": "0",
        "uploadinfos": "11",
        "romasso": "0",
        "uploadmedia": "25",
        "propositionok": "36",
        "propositionko": "5",
        "quotarefu": "13.89",
        "maxthreads": "3",
        "maxdownloadspeed": "2048",
        "requeststoday": "0",
        "requestskotoday": "0",
        "maxrequestspermin": "4096",
        "maxrequestsperday": "20000",
        "maxrequestskoperday": "2000",
        "visites": "177",
        "datedernierevisite": "2026-04-17 22:18:28",
        "favregion": "fr"
    }"#;

  const JEU: &str = r#"{
        "id": "3",
        "romid": "119923",
        "notgame": "false",
        "noms": [
            {"region": "ss",  "text": "Sonic The Hedgehog 2"},
            {"region": "wor", "text": "Sonic The Hedgehog 2"},
            {"region": "us",  "text": "Sonic The Hedgehog 2"},
            {"region": "fr",  "text": "Sonic The Hedgehog 2"}
        ],
        "systeme": {"id": "1", "text": "Megadrive"},
        "editeur": {"id": "3", "text": "SEGA"},
        "developpeur": {"id": "3", "text": "SEGA"},
        "joueurs": {"text": "2"},
        "note": {"text": "18"},
        "topstaff": "0",
        "rotation": "0",
        "synopsis": [
            {"langue": "fr", "text": "Sonic et Tails s'affrontent contre Robotnik."},
            {"langue": "en", "text": "Sonic and Tails face Robotnik."}
        ],
        "classifications": [
            {"type": "PEGI", "text": "3"}
        ],
        "dates": [
            {"region": "wor", "text": "1992-11-21"},
            {"region": "us",  "text": "1992-11-24"}
        ],
        "genres": [
            {
                "id": "4",
                "nomcourt": "Plates-formes",
                "principale": "1",
                "parentid": "0",
                "noms": [
                    {"langue": "fr", "text": "Plates-formes"},
                    {"langue": "en", "text": "Platform"}
                ]
            }
        ],
        "medias": [
            {
                "type": "mixrbv1",
                "parent": "jeu",
                "url": "https://media.screenscraper.fr/medias/1/mixrbv1.png",
                "region": "wor",
                "crc": "AABBCCDD",
                "md5": "abc123",
                "sha1": "def456",
                "size": "204800",
                "format": "png"
            },
            {
                "type": "screenshot",
                "parent": "jeu",
                "url": "https://media.screenscraper.fr/medias/1/screenshot.png",
                "crc": "11223344",
                "md5": "fff000",
                "sha1": "eee111",
                "size": "102400",
                "format": "png"
            }
        ],
        "roms": []
    }"#;

  const SYSTEME: &str = r#"{
        "id": "1",
        "parentid": "0",
        "noms": {
            "nom_ss": "Megadrive",
            "nom_eu": "Megadrive / Genesis",
            "nom_us": "Genesis",
            "nom_recalbox": "megadrive",
            "nom_retropie": "megadrive",
            "noms_commun": ["MD", "Genesis"]
        },
        "extensions": "bin,gen,md,smd,zip",
        "compagnie": "SEGA",
        "type": "Console",
        "datedebut": "1988",
        "datefin": "1997"
    }"#;

  // --- parse_user_info ---

  #[test]
  fn parse_user_info_success() {
    let body = format!(r#"{{"header": {HEADER_OK}, "response": {{"ssuser": {SSUSER}}}}}"#);
    let result = parse_user_info(&body);
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    let user = result.unwrap();
    assert_eq!(user.id, "testuser");
    assert_eq!(user.niveau, "11");
    assert_eq!(user.favregion, "fr");
    assert_eq!(user.maxthreads, 3);
    assert_eq!(user.maxdownloadspeed, 2048);
    assert_eq!(user.requeststoday, Some(0));
    assert_eq!(user.maxrequestsperday, Some(20000));
  }

  #[test]
  fn parse_user_info_api_error() {
    let body = format!(r#"{{"header": {HEADER_ERR}}}"#);
    let result = parse_user_info(&body);
    assert!(matches!(result, Err(Error::Api { .. })));
    if let Err(Error::Api { message }) = result {
      assert!(message.contains("Acces WS non autorise"));
    }
  }

  #[test]
  fn parse_user_info_missing_response() {
    let body = format!(r#"{{"header": {HEADER_OK}, "response": null}}"#);
    let result = parse_user_info(&body);
    assert!(matches!(result, Err(Error::Api { .. })));
  }

  #[test]
  fn parse_user_info_invalid_json() {
    let result = parse_user_info("not json at all");
    assert!(matches!(result, Err(Error::Parse { .. })));
  }

  // --- parse_jeu_info ---

  #[test]
  fn parse_jeu_info_success() {
    let body = format!(r#"{{"header": {HEADER_OK}, "response": {{"jeu": {JEU}}}}}"#);
    let result = parse_jeu_info(&body);
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    let jeu = result.unwrap();
    assert_eq!(jeu.id, "3");
    assert_eq!(jeu.noms.len(), 4);
    assert_eq!(jeu.systeme.as_ref().unwrap().text, "Megadrive");
    assert_eq!(jeu.medias.len(), 2);
  }

  #[test]
  fn parse_jeu_info_api_error() {
    let body = format!(r#"{{"header": {HEADER_ERR}}}"#);
    let result = parse_jeu_info(&body);
    assert!(matches!(result, Err(Error::Api { .. })));
  }

  #[test]
  fn parse_jeu_info_invalid_json() {
    let result = parse_jeu_info("{broken");
    assert!(matches!(result, Err(Error::Parse { .. })));
  }

  // --- parse_systems_liste ---

  #[test]
  fn parse_systems_liste_success() {
    let body = format!(r#"{{"header": {HEADER_OK}, "response": {{"systemes": [{SYSTEME}]}}}}"#);
    let result = parse_systems_liste(&body);
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    let systems = result.unwrap();
    assert_eq!(systems.len(), 1);
    let s = &systems[0];
    assert_eq!(s.id, "1");
    assert_eq!(s.name(), "Megadrive");
    assert_eq!(s.company.as_deref(), Some("SEGA"));
    assert!(s.extensions().contains(&"zip"));
    assert!(s.extensions().contains(&"bin"));
  }

  #[test]
  fn parse_systems_liste_api_error() {
    let body = format!(r#"{{"header": {HEADER_ERR}}}"#);
    let result = parse_systems_liste(&body);
    assert!(matches!(result, Err(Error::Api { .. })));
  }

  // --- parse_jeu_recherche ---

  #[test]
  fn parse_jeu_recherche_success() {
    let body = format!(r#"{{"header": {HEADER_OK}, "response": {{"jeux": [{JEU}]}}}}"#);
    let result = parse_jeu_recherche(&body);
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    let jeux = result.unwrap();
    assert_eq!(jeux.len(), 1);
    assert_eq!(jeux[0].id, "3");
  }

  #[test]
  fn parse_jeu_recherche_empty() {
    let body = format!(r#"{{"header": {HEADER_OK}, "response": {{"jeux": []}}}}"#);
    let result = parse_jeu_recherche(&body);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
  }

  /// ScreenScraper says "nothing found" with a list of one empty object, HTTP 200
  /// and success=true. That used to be a `missing field id` on the whole call, so
  /// an `ApiFailure::Malformed` for the caller — and, for rompom, a ROM that could
  /// no longer be identified by hand at all.
  #[test]
  fn parse_jeu_recherche_no_result_is_an_empty_list() {
    let body = format!(r#"{{"header": {HEADER_OK}, "response": {{"jeux": [{{}}]}}}}"#);
    let result = parse_jeu_recherche(&body);
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    assert!(result.unwrap().is_empty());
  }

  /// A non-empty entry that `JeuInfo` refuses stays an error: that is a schema
  /// divergence, not an API convention.
  #[test]
  fn parse_jeu_recherche_incomplete_entry_still_fails() {
    let jeu = r#"{"romid": "12"}"#;
    let body = format!(r#"{{"header": {HEADER_OK}, "response": {{"jeux": [{jeu}]}}}}"#);
    let result = parse_jeu_recherche(&body);
    assert!(matches!(result, Err(Error::Parse { .. })));
  }

  #[test]
  fn parse_jeu_recherche_api_error() {
    let body = format!(r#"{{"header": {HEADER_ERR}}}"#);
    let result = parse_jeu_recherche(&body);
    assert!(matches!(result, Err(Error::Api { .. })));
  }

  // --- HTTP errors ---

  #[test]
  fn truncated_caps_long_bodies() {
    assert_eq!(truncated(&"x".repeat(1000)).len(), BODY_MAX);
    assert_eq!(truncated("short"), "short");
  }

  #[test]
  fn truncated_does_not_split_multibyte_chars() {
    let body = "é".repeat(1000);
    let got = truncated(&body);
    assert_eq!(got.chars().count(), BODY_MAX);
  }

  #[test]
  fn http_error_display_leaks_no_credentials() {
    let err = Error::Http {
      url: "https://www.screenscraper.fr/api2/jeuInfos.php".to_string(),
      status: 430,
      body: "Votre quota de scrape est dépassé pour aujourd'hui !".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("jeuInfos.php"), "got: {}", msg);
    assert!(msg.contains("430"), "got: {}", msg);
    assert!(!msg.contains("devpassword"), "got: {}", msg);
    assert!(!msg.contains("sspassword"), "got: {}", msg);
  }
}
