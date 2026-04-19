# screenscraper

Unofficial Rust library for [ScreenScraper.fr](https://www.screenscraper.fr).

## Features

- Authenticate and fetch user info
- Look up game metadata by system ID, filename, size, and checksums (CRC32 / MD5 / SHA1)
- Rich game data: names, descriptions, genres, dates, ratings, regions, players
- Select the best available media asset per type, automatically prioritising the ROM's own region
- Download media files directly from the CDN with SHA1 verification

## Usage

### Look up a game

```rust
use screenscraper::ScreenScraper;

let ss = ScreenScraper::new(user_login, user_password, dev_login, dev_password)?;

let jeu = ss.jeuinfo(
    53,                        // system ID (e.g. 53 = Atomiswave)
    "dolphin.zip",             // ROM filename
    1234567,                   // file size in bytes
    Some("AABBCCDD".into()),   // CRC32 (optional)
    None,                      // MD5 (optional)
    None,                      // SHA1 (optional)
)?;

// Find game name with region priority
let name = jeu.find_name(&["fr", "eu", "en", "us", "wor", "jp", "ss"]);

// Get a specific media asset (best region match)
if let Some(media) = jeu.media("video-normalized").or_else(|| jeu.media("video")) {
    println!("Video URL: {}", media.url);
    println!("SHA1: {}", media.sha1);
}
```

### Download a media file

Media files are served from the ScreenScraper CDN. The `url` field of each `Media`
struct contains the direct download URL. A specific `Referer` header is required —
the library handles this automatically.

```rust
use screenscraper::ScreenScraper;
use std::path::Path;

let ss = ScreenScraper::new(user_login, user_password, dev_login, dev_password)?;
let jeu = ss.jeuinfo(53, "dolphin.zip", 1234567, None, None, None)?;

if let Some(media) = jeu.media("ss") {
    let dl = ss.media_download(&media);
    let dest = Path::new("screenshot.png");

    // Skip if already downloaded and valid
    if !dest.exists() || dl.verify_sha1(dest).is_err() {
        dl.fetch(dest)?;
    }
}
```

### Available media types

Common values for `JeuInfo::media(name)`:

| Name               | Description          |
|--------------------|----------------------|
| `ss`               | Screenshot           |
| `sstitle`          | Title screen         |
| `box-2D`           | Box art (front)      |
| `video`            | Video                |
| `video-normalized` | Normalized video     |
| `marquee`          | Marquee / logo       |
| `wheel`            | Wheel art            |
| `bezel-16-9`       | Bezel (16:9)         |
| `manuel`           | Manual (PDF)         |

## Changelog

### 0.5.0

- **`JeuInfo::media(name)`** — region selection is now ROM-aware: the method first tries the
  regions declared in `rom.regions.regions_shortname`, then falls back to
  `["wor", "ss", "eu", "us", "fr", "jp"]`. Previously the priority was fixed to
  `["fr", "eu", "us", "wor", "jp", "ss"]` regardless of the ROM's actual region.

### 0.4.0

- **`download` module added** — `MediaDownload` struct with CDN fetching and SHA1 verification.
  The required `Referer: https://screenscraper.fr/membreinfos.php` header is set automatically.
- **`ScreenScraper::media_download(&media)`** — constructs a `MediaDownload` bound to the
  library's internal HTTP client, avoiding the need to manage a separate client.

### 0.3.0

- **`JeuInfo::media(name)`** — returns the best `Media` for a given type, using a fixed
  region priority (`["fr", "eu", "us", "wor", "jp", "ss"]`). Replaces manual iteration
  over `jeu.medias`.
- `rom.regions` field added (`Option<RomRegions>`) — replaces the old `romregions` field.
  `RomRegions` exposes `regions_shortname: Vec<String>`.
- Unit tests added for `find_name`, `find_desc`, `find_date`, `find_genre`, `media`.

### 0.2.x and earlier

Initial releases — basic API authentication and `jeuInfo` lookup.
