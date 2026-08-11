use sha1::{Digest, Sha1};
use snafu::{ResultExt, Snafu};
use std::{
  fs::File,
  io::{Read, Write},
  path::{Path, PathBuf},
};

use crate::jeuinfo::Media;

const REFERER: &str = "https://screenscraper.fr/membreinfos.php";

#[derive(Debug, Snafu)]
pub enum Error {
  #[snafu(display("Failed to download {}: {}", url, source))]
  Download { url: String, source: reqwest::Error },

  #[snafu(display("Failed to read the body of {}: {}", url, source))]
  Body { url: String, source: std::io::Error },

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
    self.fetch_with_progress(dest, |_, _| {})
  }

  /// Same as [`fetch`](Self::fetch), reporting how much has been written so far.
  ///
  /// `progress(read, total)` is called once before the first byte is read (so the
  /// caller learns `total` right away), then after every chunk of 64 KiB. `total`
  /// is the `Content-Length` of the response, `None` when the server does not
  /// send one. The callback runs on the calling thread.
  pub fn fetch_with_progress(
    &self,
    dest: &Path,
    mut progress: impl FnMut(u64, Option<u64>),
  ) -> Result<()> {
    let mut res = self
      .client
      .get(&self.media.url)
      .header("Referer", REFERER)
      .send()
      .and_then(|r| r.error_for_status())
      .context(DownloadSnafu {
        url: &self.media.url,
      })?;
    let total = res.content_length();
    let mut file = File::create(dest).context(IoSnafu { path: dest })?;
    let mut buffer = vec![0u8; 64 * 1024];
    let mut read = 0u64;

    progress(read, total);
    loop {
      let n = res.read(&mut buffer).context(BodySnafu {
        url: &self.media.url,
      })?;
      if n == 0 {
        break;
      }
      file
        .write_all(&buffer[..n])
        .context(IoSnafu { path: dest })?;
      read += n as u64;
      progress(read, total);
    }
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
