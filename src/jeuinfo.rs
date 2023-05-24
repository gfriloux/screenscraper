use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;
use snafu::{ResultExt, Snafu};
use super::header::Header;

#[derive(Serialize, Deserialize, Debug)]
pub struct GenericRegionText {
   pub region:          String,
   pub text:            String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenericIdText {
   pub id:              String,
   pub text:            String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenericText {
   pub text:            String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenericLangueText {
   pub langue:          String,
   pub text:            String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Classification {
   #[serde(rename = "type")]
   pub name:            String,
   pub text:            String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenericObject {
   pub id:              String,
   pub principale:      Option<String>,
   pub parentid:        Option<String>,
   pub noms:            Option<Vec<GenericLangueText>>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Media {
   #[serde(rename = "type")]
   pub name:            String,
   pub parent:          String,
   pub url:             String,
   pub region:          Option<String>,
   pub crc:             String,
   pub md5:             String,
   pub sha1:            String,
   pub format:          String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Rom {
   pub id:              Option<String>,
   pub romsize:         Option<String>,
   pub romfilename:     String,
   pub romregions:      Option<String>,
   pub romnumsupport:   Option<String>,
   pub romtotalsupport: Option<String>,
   pub romcloneof:      String,
   pub romcrc:          Option<String>,
   pub rommd5:          Option<String>,
   pub romsha1:         Option<String>,
   pub beta:            String,
   pub demo:            String,
   pub proto:           String,
   pub trad:            String,
   pub hack:            String,
   pub unl:             String,
   pub alt:             String,
   pub best:            String,
   pub netplay:         String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JeuInfo {
   pub id:              String,
   pub romid:           Option<String>,
   pub notgame:         Option<String>,
   pub noms:            Vec<GenericRegionText>,
   pub systemeid:       Option<String>,
   pub systemenom:      Option<String>,
   pub editeur:         Option<GenericIdText>,
   pub developpeur:     Option<GenericIdText>,
   pub joueurs:         Option<GenericText>,
   pub note:            Option<GenericText>,
   pub topstaff:        String,
   pub rotation:        String,
   pub synopsis:        Option<Vec<GenericLangueText>>,
   pub classifications: Option<Vec<Classification>>,
   pub dates:           Option<Vec<GenericRegionText>>,
   pub genres:          Option<Vec<GenericObject>>,
   pub modes:           Option<Vec<GenericObject>>,
   pub familles:        Option<Vec<GenericObject>>,
   pub styles:          Option<Vec<GenericObject>>,
   pub medias:          Vec<Media>,
   pub roms:            Option<Vec<Rom>>,
   pub rom:             Option<Rom>
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseJeuInfo
{
	pub jeu: JeuInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfosResult {
   pub header:   Header,
   pub response: ResponseJeuInfo
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

impl JeuInfo {
	pub fn new(query: &Vec<(&str, String)>) -> Result<JeuInfo> {
		let client = reqwest::blocking::Client::new();
		let url = "https://www.screenscraper.fr/api2/jeuInfos.php";

      	let res = client.get(url)
      	                .query(&query)
      	                .send()
      		            .context(DownloadFailedSnafu { filename: PathBuf::from(&url) })?;
      	let s = res.text().context(DownloadFailedSnafu { filename: PathBuf::from(&url) })?;
      	println!("{}", s);
      	let response: UserInfosResult = serde_json::from_str(&s).context(ParseFailedSnafu)?;
		let user_info = response.response.jeu;
		Ok(user_info)
	}
}
