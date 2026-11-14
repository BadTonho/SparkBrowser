//! The search engines the address bar can use. Ids are stored in settings and backups, so they
//! never change. The front end keeps a matching list in `src/lib/services/search-engines.js`;
//! `tests/search-engines.test.mjs` fails when the two drift apart.

pub struct Engine {
    pub id: &'static str,
    pub label: &'static str,
    /// Landing page, used for new tabs and the home button.
    pub home: &'static str,
    /// Results page; the query goes into `param`.
    pub search: &'static str,
    pub param: &'static str,
}

pub const DEFAULT_ID: &str = "google";

pub const ENGINES: &[Engine] = &[
    Engine {
        id: "google",
        label: "Google",
        home: "https://www.google.com/",
        search: "https://www.google.com/search",
        param: "q",
    },
    Engine {
        id: "duckduckgo",
        label: "DuckDuckGo",
        home: "https://duckduckgo.com/",
        search: "https://duckduckgo.com/",
        param: "q",
    },
    Engine {
        id: "bing",
        label: "Bing",
        home: "https://www.bing.com/",
        search: "https://www.bing.com/search",
        param: "q",
    },
    Engine {
        id: "brave",
        label: "Brave Search",
        home: "https://search.brave.com/",
        search: "https://search.brave.com/search",
        param: "q",
    },
    Engine {
        id: "startpage",
        label: "Startpage",
        home: "https://www.startpage.com/",
        search: "https://www.startpage.com/do/search",
        param: "q",
    },
    Engine {
        id: "ecosia",
        label: "Ecosia",
        home: "https://www.ecosia.org/",
        search: "https://www.ecosia.org/search",
        param: "q",
    },
    Engine {
        id: "qwant",
        label: "Qwant",
        home: "https://www.qwant.com/",
        search: "https://www.qwant.com/",
        param: "q",
    },
    Engine {
        id: "yahoo",
        label: "Yahoo",
        home: "https://search.yahoo.com/",
        search: "https://search.yahoo.com/search",
        param: "p",
    },
    Engine {
        id: "mojeek",
        label: "Mojeek",
        home: "https://www.mojeek.com/",
        search: "https://www.mojeek.com/search",
        param: "q",
    },
    Engine {
        id: "kagi",
        label: "Kagi",
        home: "https://kagi.com/",
        search: "https://kagi.com/search",
        param: "q",
    },
];

pub fn find(id: &str) -> Option<&'static Engine> {
    ENGINES.iter().find(|engine| engine.id == id)
}

pub fn is_valid(id: &str) -> bool {
    find(id).is_some()
}

/// Unknown ids (a stale setting, a corrupted value) fall back to the default engine.
pub fn get_or_default(id: &str) -> &'static Engine {
    find(id).unwrap_or_else(|| find(DEFAULT_ID).expect("the default engine exists"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn ids_are_unique_and_every_engine_has_secure_urls() {
        let mut seen = HashSet::new();
        for engine in ENGINES {
            assert!(seen.insert(engine.id), "duplicate id {}", engine.id);
            assert!(!engine.label.is_empty() && !engine.param.is_empty());
            for raw in [engine.home, engine.search] {
                let url = url::Url::parse(raw).unwrap();
                assert_eq!(url.scheme(), "https", "{}", engine.id);
                assert!(url.query().is_none() && url.host_str().is_some());
            }
        }
        assert!(is_valid(DEFAULT_ID));
    }

    #[test]
    fn unknown_ids_fall_back_to_the_default() {
        assert_eq!(get_or_default("nope").id, DEFAULT_ID);
        assert_eq!(get_or_default("").id, DEFAULT_ID);
        assert_eq!(get_or_default("bing").id, "bing");
        assert!(!is_valid("GOOGLE") && !is_valid("bing "));
    }
}
