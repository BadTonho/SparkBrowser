//! Network-rule snapshots and last-known-good updates. No browsing data is sent.
use adblock::{
    engine::Engine,
    lists::{FilterSet, ParseOptions, RuleTypes},
    request::Request,
};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::Duration,
};

const SOURCES: [&str; 3] = [
    "https://easylist.to/easylist/easylist.txt",
    "https://easylist.to/easylist/easyprivacy.txt",
    "https://raw.githubusercontent.com/hoshsadiq/adblock-nocoin-list/master/nocoin.txt",
];
const MAX_SIZE: u64 = 5 * 1024 * 1024;
const REFRESH_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

/// Time left before the cached lists are due for refresh. A missing or unreadable
/// cache means the lists are due now, so the first refresh starts immediately.
fn refresh_delay(cache: &Path) -> Duration {
    std::fs::metadata(cache)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| modified.elapsed().ok())
        .map_or(Duration::ZERO, |age| REFRESH_INTERVAL.saturating_sub(age))
}

#[derive(Serialize, Deserialize)]
struct Snapshots {
    lists: [String; 3],
}

impl Snapshots {
    fn bundled() -> Self {
        Self {
            lists: [
                include_str!("../../resources/filterlists/easylist.txt").into(),
                include_str!("../../resources/filterlists/easyprivacy.txt").into(),
                include_str!("../../resources/filterlists/nocoin.txt").into(),
            ],
        }
    }
    fn validate(&self) -> Result<(), String> {
        for (index, list) in self.lists.iter().enumerate() {
            if list.len() as u64 > MAX_SIZE
                || !list.trim_start_matches('\u{feff}').starts_with("[Adblock")
            {
                return Err("Invalid filter list header or size".into());
            }
            let minimum = if index == 2 { 20 } else { 1000 };
            let count = list
                .lines()
                .filter(|line| {
                    !line.is_empty()
                        && !line.starts_with(['!', '['])
                        && adblock::filters::network::NetworkFilter::parse(
                            line,
                            false,
                            ParseOptions::default(),
                        )
                        .is_ok()
                })
                .take(minimum)
                .count();
            if count < minimum {
                return Err("Filter list is incomplete or contains no usable rules".into());
            }
        }
        Ok(())
    }
    fn compile(&self) -> Engine {
        let mut set = FilterSet::new(false);
        for list in &self.lists {
            set.add_filter_list(
                list.clone(),
                ParseOptions {
                    rule_types: RuleTypes::NetworkOnly,
                    ..Default::default()
                },
            );
        }
        Engine::new_with_filter_set(set)
    }
}

pub struct FilterLists {
    engine: RwLock<Engine>,
    cache: PathBuf,
}

impl FilterLists {
    pub fn new(cache: PathBuf) -> Self {
        let saved = std::fs::metadata(&cache)
            .ok()
            .filter(|m| m.len() <= MAX_SIZE * 3 + 4096)
            .and_then(|_| std::fs::read(&cache).ok())
            .and_then(|data| serde_json::from_slice::<Snapshots>(&data).ok())
            .filter(|s| s.validate().is_ok());
        let snapshots = saved.unwrap_or_else(Snapshots::bundled);
        Self {
            engine: RwLock::new(snapshots.compile()),
            cache,
        }
    }
    pub fn blocks(&self, url: &str, source: &str, kind: &str, method: &str) -> bool {
        let Ok(request) = Request::new(url, source, kind, method) else {
            return false;
        };
        self.engine
            .read()
            .is_ok_and(|engine| engine.check_network_request(&request).should_block())
    }
    fn install(&self, snapshots: Snapshots) -> Result<(), String> {
        snapshots.validate()?;
        let engine = snapshots.compile();
        let parent = self.cache.parent().ok_or("Invalid cache path")?;
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        serde_json::to_writer(&mut file, &snapshots).map_err(|e| e.to_string())?;
        file.flush().map_err(|e| e.to_string())?;
        file.as_file().sync_all().map_err(|e| e.to_string())?;
        file.persist(&self.cache).map_err(|e| e.to_string())?;
        *self.engine.write().map_err(|e| e.to_string())? = engine;
        Ok(())
    }
    fn refresh(&self) -> Result<(), String> {
        let client = reqwest::blocking::Client::builder()
            .https_only(true)
            .timeout(Duration::from_secs(45))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| e.to_string())?;
        let mut lists = Vec::new();
        for source in SOURCES {
            let response = client
                .get(source)
                .send()
                .and_then(|r| r.error_for_status())
                .map_err(|e| e.to_string())?;
            let mut bytes = Vec::new();
            response
                .take(MAX_SIZE + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            if bytes.len() as u64 > MAX_SIZE {
                return Err("Filter list exceeds size limit".into());
            }
            lists.push(String::from_utf8(bytes).map_err(|e| e.to_string())?);
        }
        self.install(Snapshots {
            lists: lists.try_into().map_err(|_| "Missing filter list")?,
        })
    }
    pub fn start(cache: &Path, update: bool) -> Arc<Self> {
        let service = Arc::new(Self::new(cache.to_owned()));
        if update {
            let service = Arc::clone(&service);
            std::thread::spawn(move || {
                std::thread::sleep(refresh_delay(&service.cache));
                loop {
                    if let Err(error) = service.refresh() {
                        eprintln!("Filter-list refresh failed: {error}");
                    }
                    std::thread::sleep(REFRESH_INTERVAL);
                }
            });
        }
        service
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snapshots_are_valid_and_invalid_update_keeps_engine_and_cache() {
        let directory = tempfile::tempdir().unwrap();
        let service = FilterLists::new(directory.path().join("lists.json"));
        assert!(Snapshots::bundled().validate().is_ok());
        let before = service.blocks(
            "https://coin-hive.com/lib/coinhive.min.js",
            "https://example.com/",
            "script",
            "GET",
        );
        assert!(before);
        let invalid = Snapshots {
            lists: [
                "[Adblock Plus 2.0]\n!empty".into(),
                String::new(),
                String::new(),
            ],
        };
        assert!(service.install(invalid).is_err());
        assert_eq!(
            service.blocks(
                "https://coin-hive.com/lib/coinhive.min.js",
                "https://example.com/",
                "script",
                "GET"
            ),
            before
        );
        assert!(!service.cache.exists());
    }

    #[test]
    fn refresh_waits_for_a_fresh_cache_and_runs_immediately_without_one() {
        let directory = tempfile::tempdir().unwrap();
        assert_eq!(
            refresh_delay(&directory.path().join("missing.json")),
            Duration::ZERO
        );
        let path = directory.path().join("lists.json");
        std::fs::write(&path, b"{}").unwrap();
        assert!(refresh_delay(&path) > Duration::from_secs(23 * 60 * 60));
    }

    #[test]
    fn successful_update_survives_restart_and_bad_candidate_does_not_replace_it() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("lists.json");
        let service = FilterLists::new(path.clone());
        let mut candidate = Snapshots::bundled();
        candidate.lists[0].push_str("\n||fixture-tracker.example^\n@@||fixture-tracker.example/allow.js$script,domain=example.com\n");
        service.install(candidate).unwrap();
        assert!(service.blocks(
            "https://fixture-tracker.example/block.js",
            "https://example.com/",
            "script",
            "GET"
        ));
        assert!(!service.blocks(
            "https://fixture-tracker.example/allow.js",
            "https://example.com/",
            "script",
            "GET"
        ));
        let cached = std::fs::read(&path).unwrap();
        assert!(service
            .install(Snapshots {
                lists: ["bad".into(), "bad".into(), "bad".into()]
            })
            .is_err());
        assert_eq!(std::fs::read(&path).unwrap(), cached);
        let restored = FilterLists::new(path);
        assert!(restored.blocks(
            "https://fixture-tracker.example/block.js",
            "https://example.com/",
            "script",
            "GET"
        ));
        assert!(!restored.blocks(
            "https://fixture-tracker.example/allow.js",
            "https://example.com/",
            "script",
            "GET"
        ));
    }
}
