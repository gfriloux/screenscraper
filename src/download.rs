use sha1::{Digest, Sha1};
use snafu::{ResultExt, Snafu};
use std::{
  fs::File,
  io::Read,
  path::{Path, PathBuf},
};

use crate::jeuinfo::Media;

const REFERER: &str = "https://screenscraper.fr/membreinfos.php";

#[derive(Debug, Snafu)]
pub enum Error {
  #[snafu(display("Failed to download {}: {}", url, source))]
  Download { url: String, source: reqwest::Error },

  #[snafu(display("IO error on {}: {}", path.display(), source))]
  Io {
    path: PathBuf,
    source: std::io::Error,
  },

  #[snafu(display("Checksum mismatch: expected {}, got {}", expected, got))]
  ChecksumMismatch { expected: String, got: String },
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub struct MediaDownload<'a> {
  client: &'a reqwest::blocking::Client,
  media: &'a Media,
}

impl<'a> MediaDownload<'a> {
  pub fn new(client: &'a reqwest::blocking::Client, media: &'a Media) -> Self {
    MediaDownload { client, media }
  }

  pub fn fetch(&self, dest: &Path) -> Result<()> {
    let mut res = self
      .client
      .get(&self.media.url)
      .header("Referer", REFERER)
      .send()
      .and_then(|r| r.error_for_status())
      .context(DownloadSnafu {
        url: &self.media.url,
      })?;
    let mut file = File::create(dest).context(IoSnafu { path: dest })?;
    res.copy_to(&mut file).context(DownloadSnafu {
      url: &self.media.url,
    })?;
    Ok(())
  }

  pub fn verify_sha1(&self, dest: &Path) -> Result<()> {
    let expected = &self.media.sha1;
    let mut file = File::open(dest).context(IoSnafu { path: dest })?;
    let mut hasher = Sha1::new();
    let mut buffer = [0u8; 8192];
    loop {
      let n = file.read(&mut buffer).context(IoSnafu { path: dest })?;
      if n == 0 {
        break;
      }
      hasher.update(&buffer[..n]);
    }
    let got = format!("{:x}", hasher.finalize());
    if got != *expected {
      return Err(Error::ChecksumMismatch {
        expected: expected.clone(),
        got,
      });
    }
    Ok(())
  }
}
