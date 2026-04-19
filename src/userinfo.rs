use serde::{Deserialize, Deserializer, Serialize};

fn parse_str_u32<'de, D>(d: D) -> Result<u32, D::Error>
where
  D: Deserializer<'de>,
{
  let s: String = String::deserialize(d)?;
  s.parse::<u32>().map_err(serde::de::Error::custom)
}

fn parse_opt_str_u32<'de, D>(d: D) -> Result<Option<u32>, D::Error>
where
  D: Deserializer<'de>,
{
  let opt: Option<String> = Option::deserialize(d)?;
  match opt {
    None => Ok(None),
    Some(s) if s.is_empty() => Ok(None),
    Some(s) => s.parse::<u32>().map(Some).map_err(serde::de::Error::custom),
  }
}

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
  #[serde(deserialize_with = "parse_str_u32")]
  pub maxthreads: u32,
  #[serde(deserialize_with = "parse_str_u32")]
  pub maxdownloadspeed: u32,
  #[serde(default, deserialize_with = "parse_opt_str_u32")]
  pub requeststoday: Option<u32>,
  #[serde(default, deserialize_with = "parse_opt_str_u32")]
  pub requestskotoday: Option<u32>,
  #[serde(default, deserialize_with = "parse_opt_str_u32")]
  pub maxrequestspermin: Option<u32>,
  #[serde(default, deserialize_with = "parse_opt_str_u32")]
  pub maxrequestsperday: Option<u32>,
  #[serde(default, deserialize_with = "parse_opt_str_u32")]
  pub maxrequestskoperday: Option<u32>,
  pub visites: String,
  pub datedernierevisite: String,
  pub favregion: String,
}
