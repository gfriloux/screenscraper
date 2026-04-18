use super::header::Header;
use super::jeuinfo::JeuInfo;
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

#[derive(Debug, Snafu)]
pub enum Error {
  #[snafu(display("Failed to reach API at {}: {}", url, source))]
  Request { url: String, source: reqwest::Error },

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

fn get(client: &reqwest::blocking::Client, url: &str, query: &[(&str, String)]) -> Result<String> {
  client
    .get(url)
    .query(query)
    .send()
    .and_then(|r| r.text())
    .context(RequestSnafu { url })
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
    assert_eq!(user.maxthreads, "3");
    assert_eq!(user.requeststoday, Some("0".to_string()));
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
}
