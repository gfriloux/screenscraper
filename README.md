# screenscraper

Unofficial Rust library for [ScreenScraper.fr](https://www.screenscraper.fr).

## Features

- Authenticate and fetch user info (quota, thread limit, download speed)
- Look up game metadata by system ID, filename, size, and checksums (CRC32 / MD5 / SHA1)
- Look up game metadata by ScreenScraper game ID (no ROM hashes needed)
- Search games by name (`jeuRecherche`) — up to 30 results ranked by relevance
- Fetch the full list of systems known to ScreenScraper
- Rich game data: names, descriptions, genres, dates, ratings, regions, players
- Select the best available media asset per type, automatically prioritising the ROM's own region
- Download media files directly from the CDN with SHA1 verification
- Tell failures apart: "game unknown" vs quota exceeded, thread limit, server busy…

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

Videos (`video-normalized`) run to several tens of megabytes, so `fetch()` has a
reporting counterpart. `progress(read, total)` fires once before the first byte —
that first call is how the caller learns `total` — then after every 64 KiB chunk.
`total` is `None` when the server sends no `Content-Length`:

```rust
dl.fetch_with_progress(dest, |read, total| match total {
    Some(total) => println!("{read} / {total}"),
    None => println!("{read}"),
})?;
```

### Tell failures apart

ScreenScraper signals its errors through the HTTP status code. `Error::failure()` turns
that into an `ApiFailure`, so a ROM the site does not know can be distinguished from a
quota that ran out:

```rust
use screenscraper::{ApiFailure, ScreenScraper};

match ss.jeuinfo(1, "dolphin.zip", 1234567, None, None, None) {
    Ok(jeu) => { /* ... */ }
    Err(e) if e.is_not_found() => {
        // ScreenScraper genuinely does not know this ROM — ask the user
    }
    Err(e) if e.is_retryable() => {
        // ThreadLimit (429), ServerBusy (401) or a transport error: back off and retry
    }
    Err(e) => {
        // QuotaExceeded (430), ApiClosed (423), Blacklisted (426)… over for this run
        eprintln!("{} ({:?})", e, e.failure());
    }
}
```

`is_retryable()` is `true` only for `ThreadLimit`, `ServerBusy` and `Transport`. The
library never retries nor sleeps — the backoff policy is the caller's.

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

## Security considerations

Every string field returned by this crate (`JeuInfo::find_name()`/`find_desc()`,
`Rom::romfilename`, `Media::url`/`format`, `System::name()`/`extensions()`, etc.)
is **remote, attacker-influenceable data**: ScreenScraper is a community-curated
database, and its content is not validated by this library before being handed
to you. Treat it the same way you'd treat any third-party API response.

In particular, **do not** build a filesystem path or a shell command from these
values without sanitizing/validating them first — a game name, ROM filename, or
media filename could in principle contain path-traversal sequences (`../`),
shell metacharacters, or control characters. This library performs no such
sanitization; it is the caller's responsibility.

## Changelog

### 0.8.1

- **A search with no result no longer fails the whole call.** `jeuRecherche.php` answers
  "nothing found" with HTTP 200, `success: "true"` and `jeux` holding a *single empty
  object* — not an empty list, not a 404. That empty entry made the whole response fail on
  `missing field id`, so the caller got `ApiFailure::Malformed`, indistinguishable from a
  corrupt response. `jeu_recherche()` now returns `Ok(vec![])`. Empty objects are the only
  thing filtered out: a non-empty entry `JeuInfo` refuses is a schema divergence and still
  fails the call.

### 0.8.0

- **`MediaDownload::fetch_with_progress(dest, progress)`** — same download as `fetch()`,
  calling `progress(read, total)` once before the first byte and then every 64 KiB.
  `total` comes from `Content-Length` and is `None` when the server omits it. `fetch()`
  now delegates to it with a no-op callback.
- **`download::Error::Body`** — a read that fails mid-body no longer surfaces as an
  `Io` error carrying the destination path; it carries the media URL instead.

### 0.7.0

- **HTTP status codes are no longer swallowed.** ScreenScraper reports its errors through
  the HTTP status, and an error body is not JSON — a 404 and a 430 used to surface as the
  same `Parse` error. `api::Error` gained an `Http { url, status, body }` variant; the body
  is truncated to 200 characters and `url` is the bare endpoint, never the request URL
  (which carries `devpassword` and `sspassword`).
- **`ApiFailure`** — semantic classification of a failed call: `NotFound` (404),
  `BadRequest` (400), `ServerBusy` (401), `BadDevCredentials` (403), `ApiClosed` (423),
  `Blacklisted` (426), `ThreadLimit` (429), `QuotaExceeded` (430), `KoQuotaExceeded` (431),
  `Http(u16)` for any other status, plus `Transport`, `Malformed` and `Api`.
- **`Error::failure()`** returns that classification, walking through the
  `UserInfoFailed` / `JeuInfoFailed` / `SystemsListeFailed` / `JeuRechercheFailed` wrappers.
- **`Error::is_retryable()`** (and `ApiFailure::is_retryable()`) — `true` only for
  `ThreadLimit`, `ServerBusy` and `Transport`. Quota and closed-API failures hold for the
  rest of the run. The library classifies; it never retries nor sleeps.
- **`Error::is_not_found()`** — shorthand for `failure() == ApiFailure::NotFound`, the only
  failure meaning ScreenScraper genuinely does not know the ROM.

### 0.6.0

- **`ScreenScraper::systems_liste()`** — returns the full list of systems known to
  ScreenScraper as `Vec<System>`. Each `System` exposes `name()` (priority: `nom_ss` →
  `nom_eu` → `nom_us`) and `extensions()` (CSV string split into `Vec<&str>`).
- **`ScreenScraper::jeu_recherche(system_id, name)`** — searches games by name, up to 30
  results ranked by probability. `system_id` is optional and narrows the search to one system.
  Returns `Vec<JeuInfo>`.
- **`ScreenScraper::jeuinfo_by_gameid(system_id, game_id)`** — fetches game info by
  ScreenScraper game ID, without requiring ROM hashes. ROM-specific fields will be absent.
- **`UserInfo.maxthreads` / `maxdownloadspeed`** are now `u32` (previously `String`).
  Quota fields (`requeststoday`, `maxrequestsperday`, etc.) are now `Option<u32>`.

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
