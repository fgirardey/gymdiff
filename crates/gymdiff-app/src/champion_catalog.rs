//! Champion names and square images from Riot's Data Dragon CDN, cached under the app's data
//! folder (ADR-0004). Never blocks the screen: until an image is downloaded the History shows
//! the champion's initials, and the app works fully offline.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use anyhow::Context;
use serde::{Deserialize, Serialize};

/// What the screens know of a champion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChampionInfo {
    pub name: String,
    /// The cached square image, when it is downloaded.
    pub image: Option<PathBuf>,
}

/// Names and images of the champions, shared by clones.
#[derive(Debug, Clone)]
pub struct ChampionCatalog {
    dir: PathBuf,
    state: Arc<Mutex<Cached>>,
    /// One refresh at a time: two of them would download the same image to the same file.
    refreshing: Arc<tokio::sync::Mutex<()>>,
}

impl ChampionCatalog {
    /// Reads what is cached in `<data_dir>/champions`: names when a catalog was downloaded
    /// before, nothing otherwise.
    pub fn new(data_dir: &Path) -> Self {
        let dir = data_dir.join("champions");
        let cached = read_cached(&dir).unwrap_or_default();
        Self {
            dir,
            state: Arc::new(Mutex::new(cached)),
            refreshing: Arc::default(),
        }
    }

    /// The champion with this numeric id, `None` while the catalog is not downloaded or when it
    /// does not know it (a champion newer than the catalog).
    pub fn info(&self, champion_id: u32) -> Option<ChampionInfo> {
        let state = self.lock();
        let champion = state.champions.get(&champion_id.to_string())?;
        let image = self.image_path(&state.version, &champion.key);
        Some(ChampionInfo {
            name: champion.name.clone(),
            image: image.exists().then_some(image),
        })
    }

    /// Downloads what is missing for `champion_ids`: the catalog when it is older than Data
    /// Dragon's latest version, then each square image. Returns whether anything new is cached.
    ///
    /// # Errors
    /// The CDN cannot be reached (offline) or answers something unexpected. What was downloaded
    /// before the failure is kept.
    pub async fn refresh(&self, champion_ids: &[u32]) -> anyhow::Result<bool> {
        // A refresh that waited here finds the images of the one before it already cached.
        let _one_at_a_time = self.refreshing.lock().await;
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()?;
        let mut changed = false;

        let latest = latest_version(&http).await?;
        if self.lock().version != latest {
            let catalog = download_catalog(&http, &latest).await?;
            write_cached(&self.dir, &catalog)?;
            *self.lock() = catalog;
            changed = true;
        }

        let wanted: Vec<(String, String)> = {
            let state = self.lock();
            champion_ids
                .iter()
                .filter_map(|id| state.champions.get(&id.to_string()))
                .map(|champion| (state.version.clone(), champion.key.clone()))
                .collect()
        };
        for (version, key) in wanted {
            let path = self.image_path(&version, &key);
            if path.exists() {
                continue;
            }
            download_image(&http, &version, &key, &path).await?;
            changed = true;
        }
        Ok(changed)
    }

    fn image_path(&self, version: &str, key: &str) -> PathBuf {
        self.dir.join(version).join(format!("{key}.png"))
    }

    fn lock(&self) -> MutexGuard<'_, Cached> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// The two letters standing in for a champion whose image is not there: the first two letters of
/// its name, capitalised ("Jinx" → "JI"), `?` when nothing is known.
pub fn initials(name: Option<&str>) -> String {
    let letters: String = name
        .unwrap_or_default()
        .chars()
        .filter(|c| c.is_alphabetic())
        .take(2)
        .flat_map(char::to_uppercase)
        .collect();
    if letters.is_empty() {
        "?".to_owned()
    } else {
        letters
    }
}

/// The catalog as cached: the Data Dragon version and the champions by numeric id.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Cached {
    version: String,
    /// Keyed by the numeric id (`"222"`), as Data Dragon's `key` field gives it.
    champions: HashMap<String, Champion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Champion {
    /// Data Dragon's `id`, the file name of the image ("`MissFortune`").
    key: String,
    name: String,
}

const DATA_DRAGON: &str = "https://ddragon.leagueoflegends.com";
const CATALOG_FILE: &str = "catalog.json";

fn read_cached(dir: &Path) -> Option<Cached> {
    let text = std::fs::read_to_string(dir.join(CATALOG_FILE)).ok()?;
    serde_json::from_str(&text).ok()
}

/// Writes a temporary file then renames it, so a crash never leaves a half-written catalog.
fn write_cached(dir: &Path, catalog: &Cached) -> anyhow::Result<()> {
    std::fs::create_dir_all(dir)?;
    let temporary = dir.join(format!("{CATALOG_FILE}.tmp"));
    std::fs::write(&temporary, serde_json::to_string(catalog)?)?;
    std::fs::rename(temporary, dir.join(CATALOG_FILE))?;
    Ok(())
}

async fn latest_version(http: &reqwest::Client) -> anyhow::Result<String> {
    let versions: Vec<String> = get(http, &format!("{DATA_DRAGON}/api/versions.json"))
        .await?
        .json()
        .await
        .context("reading Data Dragon's versions")?;
    versions
        .into_iter()
        .next()
        .context("Data Dragon lists no version")
}

async fn download_catalog(http: &reqwest::Client, version: &str) -> anyhow::Result<Cached> {
    let text = get(
        http,
        &format!("{DATA_DRAGON}/cdn/{version}/data/en_US/champion.json"),
    )
    .await?
    .text()
    .await?;
    parse_catalog(version, &text)
}

/// Data Dragon's `champion.json` as a [`Cached`] catalog. Unknown fields are ignored.
fn parse_catalog(version: &str, json: &str) -> anyhow::Result<Cached> {
    #[derive(Deserialize)]
    struct File {
        data: HashMap<String, Entry>,
    }
    #[derive(Deserialize)]
    struct Entry {
        id: String,
        key: String,
        name: String,
    }
    let file: File = serde_json::from_str(json).context("reading Data Dragon's champions")?;
    Ok(Cached {
        version: version.to_owned(),
        champions: file
            .data
            .into_values()
            .map(|entry| {
                (
                    entry.key,
                    Champion {
                        key: entry.id,
                        name: entry.name,
                    },
                )
            })
            .collect(),
    })
}

async fn download_image(
    http: &reqwest::Client,
    version: &str,
    key: &str,
    path: &Path,
) -> anyhow::Result<()> {
    let bytes = get(
        http,
        &format!("{DATA_DRAGON}/cdn/{version}/img/champion/{key}.png"),
    )
    .await?
    .bytes()
    .await?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    // A temporary file first: a half-written image would never be fetched again.
    let temporary = path.with_extension("png.tmp");
    std::fs::write(&temporary, &bytes)?;
    std::fs::rename(temporary, path)?;
    Ok(())
}

/// `GET`s `url`; an error status is an error. The requests carry nothing but the path.
async fn get(http: &reqwest::Client, url: &str) -> anyhow::Result<reqwest::Response> {
    http.get(url)
        .send()
        .await
        .with_context(|| format!("requesting {url}"))?
        .error_for_status()
        .with_context(|| format!("requesting {url}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHAMPION_JSON: &str = r#"{
        "type": "champion",
        "version": "16.22.1",
        "data": {
            "Jinx": {"id": "Jinx", "key": "222", "name": "Jinx", "title": "the Loose Cannon", "tags": ["Marksman"]},
            "MissFortune": {"id": "MissFortune", "key": "21", "name": "Miss Fortune"}
        }
    }"#;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("gymdiff-champions-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn parses_data_dragons_catalog_by_numeric_id() {
        let catalog = parse_catalog("16.22.1", CHAMPION_JSON).unwrap();
        assert_eq!(catalog.version, "16.22.1");
        assert_eq!(catalog.champions["21"].key, "MissFortune");
        assert_eq!(catalog.champions["21"].name, "Miss Fortune");
        assert_eq!(catalog.champions["222"].key, "Jinx");
    }

    #[test]
    fn an_unexpected_payload_is_an_error_not_a_panic() {
        assert!(parse_catalog("1", "{}").is_err());
        assert!(parse_catalog("1", "not json").is_err());
    }

    #[test]
    fn nothing_is_known_before_a_catalog_is_cached() {
        let dir = temp_dir("empty");
        assert_eq!(ChampionCatalog::new(&dir).info(222), None);
    }

    #[test]
    fn a_cached_catalog_gives_names_and_only_the_images_that_exist() {
        let dir = temp_dir("cached");
        let champions = dir.join("champions");
        write_cached(
            &champions,
            &parse_catalog("16.22.1", CHAMPION_JSON).unwrap(),
        )
        .unwrap();

        let catalog = ChampionCatalog::new(&dir);
        assert_eq!(
            catalog.info(222),
            Some(ChampionInfo {
                name: "Jinx".to_owned(),
                image: None
            })
        );

        std::fs::create_dir_all(champions.join("16.22.1")).unwrap();
        std::fs::write(champions.join("16.22.1").join("Jinx.png"), b"png").unwrap();
        assert_eq!(
            catalog.info(222).unwrap().image,
            Some(champions.join("16.22.1").join("Jinx.png"))
        );
        assert_eq!(catalog.info(999), None);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_damaged_cache_is_ignored() {
        let dir = temp_dir("damaged");
        std::fs::create_dir_all(dir.join("champions")).unwrap();
        std::fs::write(dir.join("champions").join(CATALOG_FILE), b"{ nope").unwrap();
        assert_eq!(ChampionCatalog::new(&dir).info(222), None);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn initials_are_the_first_two_letters() {
        assert_eq!(initials(Some("Jinx")), "JI");
        assert_eq!(initials(Some("Miss Fortune")), "MI");
        assert_eq!(initials(Some("K'Sante")), "KS");
        assert_eq!(initials(Some("")), "?");
        assert_eq!(initials(None), "?");
    }
}
