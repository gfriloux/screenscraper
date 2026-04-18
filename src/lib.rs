mod api;
pub mod download;
mod header;
pub mod jeuinfo;
pub mod userinfo;

use crate::jeuinfo::JeuInfo;
use crate::userinfo::UserInfo;
#[cfg(test)]
use dotenv::dotenv;
use snafu::{ResultExt, Snafu};

#[derive(Debug)]
pub struct ScreenScraper {
  pub user_login: String,
  pub dev_login: String,
  pub soft_name: String,
  pub user_info: Option<UserInfo>,
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
}

type Result<T, E = Error> = std::result::Result<T, E>;

impl ScreenScraper {
  pub fn new(
    user_login: &str,
    user_password: &str,
    dev_login: &str,
    dev_password: &str,
  ) -> Result<ScreenScraper> {
    let mut ss = ScreenScraper {
      user_login: user_login.to_string(),
      user_password: user_password.to_string(),
      dev_login: dev_login.to_string(),
      dev_password: dev_password.to_string(),
      soft_name: "ScreenScraper Rust Library".to_string(),
      user_info: None,
      client: reqwest::blocking::Client::new(),
    };

    ss.load()?;
    Ok(ss)
  }

  fn load(&mut self) -> Result<()> {
    let query = self.base_query();
    self.user_info = Some(api::fetch_user_info(&self.client, &query).context(
      UserInfoFailedSnafu {
        user: self.user_login.clone(),
      },
    )?);
    Ok(())
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
  }
}
