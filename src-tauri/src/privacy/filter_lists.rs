//! Network-rule snapshots and last-known-good updates. No browsing data is sent.
use crate::sync::RwLock;
use adblock::{
    engine::Engine,
    lists::{FilterSet, ParseOptions, RuleTypes},
    request::Request,
};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
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
    /// Network rules and element-hiding rules go to the engine; scriptlets are only used once their
    /// code is supplied (see the shield documentation).
    fn compile(self) -> Engine {
        let mut set = FilterSet::new(false);
        for list in self.lists {
            set.add_filter_list(
                list,
                ParseOptions {
                    rule_types: RuleTypes::All,
                    ..Default::default()
                },
            );
        }
        Engine::new_with_filter_set(set)
    }
}

pub struct FilterLists {
    engine: RwLock<Engine>,
    /// Generic element-hiding selectors of the current lists.
    generic_hide: RwLock<Arc<super::cosmetic::GenericHide>>,
    cache: PathBuf,
    /// The lists the engine was built from, kept to derive the WebKit content rules.
    #[cfg(target_os = "linux")]
    lists: RwLock<Vec<String>>,
    /// Called after an update replaces the engine.
    #[cfg(target_os = "linux")]
    on_update: std::sync::OnceLock<Box<dyn Fn() + Send + Sync>>,
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
        let generic_hide = RwLock::new(Arc::new(super::cosmetic::GenericHide::new(
            super::cosmetic::generic_selectors(&snapshots.lists),
        )));
        #[cfg(target_os = "linux")]
        let lists = RwLock::new(snapshots.lists.to_vec());
        Self {
            engine: RwLock::new(snapshots.compile()),
            generic_hide,
            cache,
            #[cfg(target_os = "linux")]
            lists,
            #[cfg(target_os = "linux")]
            on_update: std::sync::OnceLock::new(),
        }
    }
    /// Runs `hook` after every successful list update.
    #[cfg(target_os = "linux")]
    pub fn set_on_update(&self, hook: impl Fn() + Send + Sync + 'static) {
        let _ = self.on_update.set(Box::new(hook));
    }
    /// The current network rules in the JSON format of WebKit content rule lists.
    #[cfg(target_os = "linux")]
    pub fn content_blocking_json(&self) -> Option<String> {
        let lists = self.lists.read().ok()?.clone();
        let mut set = FilterSet::new(true);
        for list in lists {
            set.add_filter_list(
                list,
                ParseOptions {
                    rule_types: RuleTypes::NetworkOnly,
                    ..Default::default()
                },
            );
        }
        let (mut rules, _) = set.into_content_blocking().ok()?;
        // WebKit rejects lists above 150000 rules; exceptions stay, they must come last.
        const LIMIT: usize = 149_000;
        if rules.len() > LIMIT {
            use adblock::content_blocking::CbType;
            let (exceptions, mut others): (Vec<_>, Vec<_>) = rules
                .into_iter()
                .partition(|rule| rule.action.typ == CbType::IgnorePreviousRules);
            others.truncate(LIMIT.saturating_sub(exceptions.len()));
            others.extend(exceptions);
            rules = others;
        }
        serde_json::to_string(&rules).ok()
    }
    /// The rules that hide ads on the page at `url`: the ones written for its site plus the generic
    /// ones, unless the lists say not to use those there.
    pub fn cosmetics(&self, url: &str) -> super::cosmetic::PageCosmetics {
        let Ok(engine) = self.engine.read() else {
            return super::cosmetic::PageCosmetics::new(&[], None, &Default::default());
        };
        let resources = engine.url_cosmetic_resources(url);
        drop(engine);
        let mut site: Vec<String> = resources
            .hide_selectors
            .iter()
            .filter(|selector| super::cosmetic::is_plain_selector(selector))
            .cloned()
            .collect();
        site.sort();
        let generic = if resources.generichide {
            None
        } else {
            self.generic_hide
                .read()
                .ok()
                .map(|generic| Arc::clone(&generic))
        };
        super::cosmetic::PageCosmetics::new(&site, generic, &resources.exceptions)
    }
    /// The whole style sheet for `url`.
    #[cfg(test)]
    pub fn cosmetic_css(&self, url: &str) -> String {
        self.cosmetics(url).css()
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
        #[cfg(target_os = "linux")]
        let lists = snapshots.lists.to_vec();
        let parent = self.cache.parent().ok_or("Invalid cache path")?;
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        serde_json::to_writer(&mut file, &snapshots).map_err(|e| e.to_string())?;
        let generic = Arc::new(super::cosmetic::GenericHide::new(
            super::cosmetic::generic_selectors(&snapshots.lists),
        ));
        let engine = snapshots.compile();
        file.flush().map_err(|e| e.to_string())?;
        file.as_file().sync_all().map_err(|e| e.to_string())?;
        file.persist(&self.cache).map_err(|e| e.to_string())?;
        *self.engine.write().map_err(|e| e.to_string())? = engine;
        *self.generic_hide.write().map_err(|e| e.to_string())? = generic;
        #[cfg(target_os = "linux")]
        {
            if let Ok(mut current) = self.lists.write() {
                *current = lists;
            }
            if let Some(hook) = self.on_update.get() {
                hook();
            }
        }
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
    pub fn start_updates(self: &Arc<Self>) {
        let service = Arc::clone(self);
        std::thread::spawn(move || {
            std::thread::sleep(refresh_delay(&service.cache));
            loop {
                if let Err(error) = service.refresh() {
                    crate::error_log::warn(&format!("Filter-list refresh failed: {error}"));
                }
                std::thread::sleep(REFRESH_INTERVAL);
            }
        });
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
    fn pages_get_their_own_hiding_rules_plus_the_generic_ones_unless_excepted() {
        let directory = tempfile::tempdir().unwrap();
        let service = FilterLists::new(directory.path().join("lists.json"));
        let mut candidate = Snapshots::bundled();
        candidate.lists[0].push_str(
            "\nshop.example##.only-on-shop\n##.everywhere-box\nquiet.example#@#.everywhere-box\n",
        );
        service.install(candidate).unwrap();
        let shop = service.cosmetic_css("https://shop.example/page");
        assert!(shop.contains(".only-on-shop") && shop.contains(".everywhere-box"));
        let other = service.cosmetic_css("https://other.example/");
        assert!(!other.contains(".only-on-shop") && other.contains(".everywhere-box"));
        assert!(!service
            .cosmetic_css("https://quiet.example/")
            .contains(".everywhere-box"));
        assert!(other.starts_with(":is(") && other.contains("display:none!important"));
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
