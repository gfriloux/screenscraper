use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenericRegionText {
  pub region: String,
  pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenericIdText {
  pub id: String,
  pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenericText {
  pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenericLangueText {
  pub langue: String,
  pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Classification {
  #[serde(rename = "type")]
  pub name: String,
  pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenericObject {
  pub id: String,
  pub nomcourt: Option<String>,
  pub principale: Option<String>,
  pub parentid: Option<String>,
  pub noms: Option<Vec<GenericLangueText>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Media {
  #[serde(rename = "type")]
  pub name: String,
  pub parent: String,
  pub url: String,
  pub region: Option<String>,
  pub crc: String,
  pub md5: String,
  pub sha1: String,
  pub size: Option<String>,
  pub format: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RomRegions {
  pub regions_id: Option<Vec<String>>,
  pub regions_shortname: Vec<String>,
  pub regions_en: Option<Vec<String>>,
  pub regions_fr: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rom {
  pub id: Option<String>,
  pub romsize: Option<String>,
  pub romfilename: String,
  pub regions: Option<RomRegions>,
  pub romnumsupport: Option<String>,
  pub romtotalsupport: Option<String>,
  pub romcloneof: String,
  pub romcrc: Option<String>,
  pub rommd5: Option<String>,
  pub romsha1: Option<String>,
  pub beta: String,
  pub demo: String,
  pub proto: String,
  pub trad: String,
  pub hack: String,
  pub unl: String,
  pub alt: String,
  pub best: String,
  pub netplay: Option<String>,
  pub retroachievement: Option<String>,
  pub gamelink: Option<String>,
  pub nbscrap: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JeuInfo {
  pub id: String,
  pub romid: Option<String>,
  pub notgame: Option<String>,
  pub noms: Vec<GenericRegionText>,
  pub systeme: Option<GenericIdText>,
  pub editeur: Option<GenericIdText>,
  pub developpeur: Option<GenericIdText>,
  pub joueurs: Option<GenericText>,
  pub note: Option<GenericText>,
  pub topstaff: String,
  pub rotation: String,
  pub synopsis: Option<Vec<GenericLangueText>>,
  pub classifications: Option<Vec<Classification>>,
  pub dates: Option<Vec<GenericRegionText>>,
  pub genres: Option<Vec<GenericObject>>,
  pub modes: Option<Vec<GenericObject>>,
  pub familles: Option<Vec<GenericObject>>,
  pub styles: Option<Vec<GenericObject>>,
  pub medias: Vec<Media>,
  pub roms: Option<Vec<Rom>>,
  pub rom: Option<Rom>,
}

impl JeuInfo {
  pub fn media(&self, name: &str) -> Option<Media> {
    let priority = ["fr", "eu", "us", "wor", "jp", "ss"];
    priority.iter().find_map(|&region| {
      self
        .medias
        .iter()
        .find(|m| m.name == name && m.region.as_deref().is_none_or(|r| r == region))
        .cloned()
    })
  }

  pub fn find_name(&self, fav: &[&str]) -> String {
    if let Some(rom) = &self.rom {
      if let Some(regions) = &rom.regions {
        for region in &regions.regions_shortname {
          if let Some(nom) = self.noms.iter().find(|n| &n.region == region) {
            return nom.text.clone();
          }
        }
      }
    }
    fav
      .iter()
      .find_map(|&region| self.noms.iter().find(|n| n.region == region))
      .map(|n| n.text.clone())
      .unwrap_or_else(|| "Unknown".to_string())
  }

  pub fn find_desc(&self, fav: &[&str]) -> String {
    self
      .synopsis
      .as_ref()
      .and_then(|synopses| {
        fav
          .iter()
          .find_map(|&lang| synopses.iter().find(|s| s.langue == lang))
      })
      .map(|s| s.text.clone())
      .unwrap_or_else(|| "Unknown".to_string())
  }

  pub fn find_date(&self, fav: &[&str]) -> String {
    self
      .dates
      .as_ref()
      .and_then(|dates| {
        fav
          .iter()
          .find_map(|&region| dates.iter().find(|d| d.region == region))
      })
      .map(|d| d.text.clone())
      .unwrap_or_else(|| "Unknown".to_string())
  }

  pub fn find_genre(&self, fav: &[&str]) -> String {
    self
      .genres
      .as_ref()
      .and_then(|genres| {
        fav.iter().find_map(|&lang| {
          genres
            .iter()
            .filter(|g| g.principale.as_deref().is_none_or(|p| p == "1"))
            .find_map(|g| g.noms.as_ref()?.iter().find(|n| n.langue == lang))
        })
      })
      .map(|n| n.text.clone())
      .unwrap_or_else(|| "Unknown".to_string())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn make_jeu(noms: Vec<GenericRegionText>, rom: Option<Rom>) -> JeuInfo {
    JeuInfo {
      id: "3".to_string(),
      romid: None,
      notgame: None,
      noms,
      systeme: None,
      editeur: None,
      developpeur: None,
      joueurs: None,
      note: None,
      topstaff: "0".to_string(),
      rotation: "0".to_string(),
      synopsis: None,
      classifications: None,
      dates: None,
      genres: None,
      modes: None,
      familles: None,
      styles: None,
      medias: vec![],
      roms: None,
      rom,
    }
  }

  fn make_nom(region: &str, text: &str) -> GenericRegionText {
    GenericRegionText {
      region: region.to_string(),
      text: text.to_string(),
    }
  }

  fn make_rom(regions_shortname: Vec<&str>) -> Rom {
    Rom {
      id: None,
      romsize: None,
      romfilename: "sonic.zip".to_string(),
      regions: Some(RomRegions {
        regions_id: None,
        regions_shortname: regions_shortname
          .into_iter()
          .map(|s| s.to_string())
          .collect(),
        regions_en: None,
        regions_fr: None,
      }),
      romnumsupport: None,
      romtotalsupport: None,
      romcloneof: "0".to_string(),
      romcrc: None,
      rommd5: None,
      romsha1: None,
      beta: "0".to_string(),
      demo: "0".to_string(),
      proto: "0".to_string(),
      trad: "0".to_string(),
      hack: "0".to_string(),
      unl: "0".to_string(),
      alt: "0".to_string(),
      best: "0".to_string(),
      netplay: None,
      retroachievement: None,
      gamelink: None,
      nbscrap: None,
    }
  }

  // --- find_name ---

  #[test]
  fn find_name_uses_rom_region_first() {
    let jeu = make_jeu(
      vec![make_nom("us", "Sonic US"), make_nom("wor", "Sonic World")],
      Some(make_rom(vec!["wor"])),
    );
    assert_eq!(jeu.find_name(&["us", "wor"]), "Sonic World");
  }

  #[test]
  fn find_name_falls_back_to_fav() {
    let jeu = make_jeu(
      vec![make_nom("us", "Sonic US"), make_nom("fr", "Sonic FR")],
      None,
    );
    assert_eq!(jeu.find_name(&["fr", "us"]), "Sonic FR");
  }

  #[test]
  fn find_name_returns_unknown_when_not_found() {
    let jeu = make_jeu(vec![make_nom("jp", "Sonic JP")], None);
    assert_eq!(jeu.find_name(&["fr", "us"]), "Unknown");
  }

  // --- find_desc ---

  #[test]
  fn find_desc_returns_preferred_language() {
    let mut jeu = make_jeu(vec![], None);
    jeu.synopsis = Some(vec![
      GenericLangueText {
        langue: "en".to_string(),
        text: "Sonic EN".to_string(),
      },
      GenericLangueText {
        langue: "fr".to_string(),
        text: "Sonic FR".to_string(),
      },
    ]);
    assert_eq!(jeu.find_desc(&["fr", "en"]), "Sonic FR");
  }

  #[test]
  fn find_desc_returns_unknown_when_no_synopsis() {
    let jeu = make_jeu(vec![], None);
    assert_eq!(jeu.find_desc(&["fr"]), "Unknown");
  }

  // --- find_date ---

  #[test]
  fn find_date_returns_preferred_region() {
    let mut jeu = make_jeu(vec![], None);
    jeu.dates = Some(vec![
      GenericRegionText {
        region: "wor".to_string(),
        text: "1992-11-21".to_string(),
      },
      GenericRegionText {
        region: "us".to_string(),
        text: "1992-11-24".to_string(),
      },
    ]);
    assert_eq!(jeu.find_date(&["us", "wor"]), "1992-11-24");
  }

  #[test]
  fn find_date_returns_unknown_when_no_dates() {
    let jeu = make_jeu(vec![], None);
    assert_eq!(jeu.find_date(&["fr"]), "Unknown");
  }

  // --- find_genre ---

  #[test]
  fn find_genre_returns_principale_genre() {
    let mut jeu = make_jeu(vec![], None);
    jeu.genres = Some(vec![GenericObject {
      id: "4".to_string(),
      nomcourt: None,
      principale: Some("1".to_string()),
      parentid: Some("0".to_string()),
      noms: Some(vec![
        GenericLangueText {
          langue: "fr".to_string(),
          text: "Plates-formes".to_string(),
        },
        GenericLangueText {
          langue: "en".to_string(),
          text: "Platform".to_string(),
        },
      ]),
    }]);
    assert_eq!(jeu.find_genre(&["fr", "en"]), "Plates-formes");
  }

  #[test]
  fn find_genre_skips_non_principale() {
    let mut jeu = make_jeu(vec![], None);
    jeu.genres = Some(vec![GenericObject {
      id: "5".to_string(),
      nomcourt: None,
      principale: Some("0".to_string()),
      parentid: None,
      noms: Some(vec![GenericLangueText {
        langue: "fr".to_string(),
        text: "Action".to_string(),
      }]),
    }]);
    assert_eq!(jeu.find_genre(&["fr"]), "Unknown");
  }

  #[test]
  fn find_genre_returns_unknown_when_no_genres() {
    let jeu = make_jeu(vec![], None);
    assert_eq!(jeu.find_genre(&["fr"]), "Unknown");
  }

  // --- media ---

  #[test]
  fn media_returns_preferred_region() {
    let mut jeu = make_jeu(vec![], None);
    jeu.medias = vec![
      Media {
        name: "screenshot".to_string(),
        parent: "jeu".to_string(),
        url: "https://example.com/us.png".to_string(),
        region: Some("us".to_string()),
        crc: "AA".to_string(),
        md5: "BB".to_string(),
        sha1: "CC".to_string(),
        size: None,
        format: "png".to_string(),
      },
      Media {
        name: "screenshot".to_string(),
        parent: "jeu".to_string(),
        url: "https://example.com/fr.png".to_string(),
        region: Some("fr".to_string()),
        crc: "DD".to_string(),
        md5: "EE".to_string(),
        sha1: "FF".to_string(),
        size: None,
        format: "png".to_string(),
      },
    ];
    // priority = ["fr", "eu", "us", ...] → fr comes first
    let m = jeu.media("screenshot").unwrap();
    assert_eq!(m.region.as_deref(), Some("fr"));
  }

  #[test]
  fn media_returns_none_when_not_found() {
    let jeu = make_jeu(vec![], None);
    assert!(jeu.media("screenshot").is_none());
  }

  #[test]
  fn media_without_region_matches() {
    let mut jeu = make_jeu(vec![], None);
    jeu.medias = vec![Media {
      name: "wheel".to_string(),
      parent: "jeu".to_string(),
      url: "https://example.com/wheel.png".to_string(),
      region: None,
      crc: "00".to_string(),
      md5: "11".to_string(),
      sha1: "22".to_string(),
      size: None,
      format: "png".to_string(),
    }];
    assert!(jeu.media("wheel").is_some());
  }
}
