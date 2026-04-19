use serde::{Deserialize, Serialize};

/// Names of a system in various locales and front-ends.
/// Unknown `nom_xx` fields from the API are silently ignored.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SystemNames {
  #[serde(rename = "nom_ss")]
  pub ss: Option<String>,
  #[serde(rename = "nom_eu")]
  pub eu: Option<String>,
  #[serde(rename = "nom_us")]
  pub us: Option<String>,
  #[serde(rename = "nom_jp")]
  pub jp: Option<String>,
  #[serde(rename = "nom_fr")]
  pub fr: Option<String>,
  #[serde(default, rename = "nom_recalbox")]
  pub recalbox: Option<String>,
  #[serde(default, rename = "nom_retropie")]
  pub retropie: Option<String>,
  #[serde(default, rename = "nom_launchbox")]
  pub launchbox: Option<String>,
  #[serde(default, rename = "nom_hyperspin")]
  pub hyperspin: Option<String>,
  #[serde(default, rename = "noms_commun")]
  pub common: Vec<String>,
}

/// A ScreenScraper system entry as returned by `systemesListe.php`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct System {
  pub id: String,
  #[serde(rename = "parentid")]
  pub parent_id: Option<String>,
  pub noms: Option<SystemNames>,
  /// Comma-separated list of supported ROM file extensions (e.g. `"bin,gen,md,zip"`).
  pub extensions: Option<String>,
  #[serde(rename = "compagnie")]
  pub company: Option<String>,
  /// System type: `Arcade`, `Console`, `Console Portable`, etc.
  #[serde(rename = "type")]
  pub type_: Option<String>,
  pub datedebut: Option<String>,
  pub datefin: Option<String>,
}

impl System {
  /// Returns the canonical name of the system, trying `nom_ss` then `nom_eu` then `nom_us`.
  pub fn name(&self) -> &str {
    self
      .noms
      .as_ref()
      .and_then(|n| n.ss.as_deref().or(n.eu.as_deref()).or(n.us.as_deref()))
      .unwrap_or("Unknown")
  }

  /// Returns the supported ROM extensions as a slice of `&str`.
  pub fn extensions(&self) -> Vec<&str> {
    self
      .extensions
      .as_deref()
      .map(|e| {
        e.split(',')
          .map(str::trim)
          .filter(|s| !s.is_empty())
          .collect()
      })
      .unwrap_or_default()
  }
}
