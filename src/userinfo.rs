use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;
use snafu::{ResultExt, Snafu};
use super::header::Header;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfo {
   pub id:                 String,
   pub niveau:             String,
   pub contribution:       String,
   pub uploadsysteme:      String,
   pub uploadinfos:        String,
   pub romasso:            String,
   pub uploadmedia:        String,
   pub maxthreads:         String,
   pub maxdownloadspeed:   String,
   pub requeststoday:      Option<String>,
   pub visites:            String,
   pub datedernierevisite: String,
   pub favregion:          String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseUserInfo
{
	pub ssuser: UserInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfosResult {
   pub header:   Header,
   pub response: ResponseUserInfo
}

#[derive(Debug, Snafu)]
pub enum Error {
   #[snafu(display("Failed to download {}: {}", filename.display(), source))]
   DownloadFailed {
      filename: PathBuf,
      source: reqwest::Error,
   },

   #[snafu(display("Failed to read received data: {}", source))]
   ParseFailed {
      source: serde_json::Error,
   },
}

type Result<T, E = Error> = std::result::Result<T, E>;

impl UserInfo {
	pub fn new(query: &Vec<(&str, String)>) -> Result<UserInfo> {
		let client = reqwest::blocking::Client::new();
		let url = "https://www.screenscraper.fr/api2/ssuserInfos.php";

      	let res = client.get(url)
      	                .query(&query)
      	                .send()
      		            .context(DownloadFailedSnafu { filename: PathBuf::from(&url) })?;
      	let s = res.text().context(DownloadFailedSnafu { filename: PathBuf::from(&url) })?;
      	let response: UserInfosResult = serde_json::from_str(&s).context(ParseFailedSnafu)?;
		let user_info = response.response.ssuser;
		Ok(user_info)
	}
}
