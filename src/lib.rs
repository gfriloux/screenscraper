extern crate reqwest;
extern crate snafu;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

#[cfg(test)] extern crate dotenv;

mod userinfo;
mod jeuinfo;
mod header;

use crate::userinfo::UserInfo;
use crate::jeuinfo::JeuInfo;
use snafu::{ResultExt, Snafu};
#[cfg(test)] use dotenv::dotenv;

#[derive(Debug)]
pub struct ScreenScraper
{
	pub user_login:         String,
	pub user_password:      String,
	pub dev_login:          String,
	pub dev_password:       String,
	pub soft_name:          String,
	pub user_info:          Option<UserInfo>
}

#[derive(Debug, Snafu)]
pub enum Error {
   #[snafu(display("Failed to fetch UserInfo for {}", user))]
   UserInfoFailed {
   	  user: String,
   	  source: userinfo::Error,
   },
   #[snafu(display("Failed to fetch UserInfo for {}", user))]
   JeuInfoFailed {
   	  user: String,
   	  source: jeuinfo::Error,
   },
}

type Result<T, E = Error> = std::result::Result<T, E>;

impl ScreenScraper {
	pub fn new(
		user_login: &str,
		user_password: &str,
		dev_login: &str,
		dev_password: &str
	) -> Result<ScreenScraper> {
		let mut ss = ScreenScraper {
			user_login: user_login.to_string(),
			user_password: user_password.to_string(),
			dev_login: dev_login.to_string(),
			dev_password: dev_password.to_string(),
			soft_name: "ScreenScraper Rust Library".to_string(),
			user_info: None
		};

		ss.load().unwrap();
		Ok(ss)
	}

	pub fn load(&mut self) -> Result<()> {
		let mut query  = Vec::new();
		query.push(("devid"      , self.dev_login.clone()));
      	query.push(("devpassword", self.dev_password.clone()));
      	query.push(("softname"   , self.soft_name.clone()));
      	query.push(("ssid"       , self.user_login.clone()));
      	query.push(("sspassword" , self.user_password.clone()));
      	query.push(("output"     , "json".to_string()));

		self.user_info = Some(UserInfo::new(&query).context(UserInfoFailedSnafu { user: self.user_login.clone() })?);
		Ok(())
	}

	pub fn jeuinfo(&self, id: u32, filename: &str, filesize: u64) -> Result<JeuInfo> {
		let mut query  = Vec::new();
		query.push(("devid"      , self.dev_login.clone()));
      	query.push(("devpassword", self.dev_password.clone()));
      	query.push(("softname"   , self.soft_name.clone()));
      	query.push(("ssid"       , self.user_login.clone()));
      	query.push(("sspassword" , self.user_password.clone()));
      	query.push(("output"     , "json".to_string()));
      	query.push(("systemeid"  , format!("{}", id)));
      	query.push(("romnom"     , filename.to_string()));
      	query.push(("romtaille"  , format!("{}", filesize)));

		Ok(JeuInfo::new(&query).context(JeuInfoFailedSnafu { user: filename.to_string() })?)
	}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    	dotenv().ok();

        let ss = ScreenScraper::new(&std::env::var("SS_USER_LOGIN").unwrap(),
        						    &std::env::var("SS_USER_PASSWORD").unwrap(),
        						    &std::env::var("SS_DEV_LOGIN").unwrap(),
        						    &std::env::var("SS_DEV_PASSWORD").unwrap()).unwrap();
        println!("{:#?}", ss);

        let ji = ss.jeuinfo(1, "Sonic The Hedgehog (World).zip", 749652).unwrap();
        println!("{:#?}", ji);
    }
}
