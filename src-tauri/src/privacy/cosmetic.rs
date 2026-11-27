//! Element hiding ("cosmetic filtering"): the `##selector` rules of the filter lists, applied as a
//! style sheet to the pages where the shield is on. Network rules cannot remove an empty ad frame or
//! a "sponsored" box, which is what these rules are for.
//!
//! Only plain CSS selectors are used. Rules that need a script to evaluate them (`:has-text(...)`,
//! `:xpath(...)`, and so on) are skipped, and so is anything that could break out of a style rule.
use std::collections::HashSet;
use std::sync::Arc;
use tauri::{Manager, Webview};

/// Selectors per `:is()` group: invalid ones inside it are ignored by the browser instead of
/// invalidating the whole rule, and grouping keeps the style sheet small.
const GROUP: usize = 120;
const MAX_SELECTOR: usize = 400;
/// Pseudo-classes that are plain CSS. Anything else is a procedural rule written for a script.
const PSEUDO: &[&str] = &[
    "not",
    "is",
    "where",
    "has",
    "nth-child",
    "nth-of-type",
    "first-child",
    "last-child",
    "first-of-type",
    "last-of-type",
    "only-child",
    "empty",
    "link",
    "visited",
    "hover",
];

/// Whether `selector` is safe to put in a style rule and needs no script to evaluate.
pub fn is_plain_selector(selector: &str) -> bool {
    if selector.is_empty()
        || selector.len() > MAX_SELECTOR
        || selector.contains(['{', '}', ';', '\\', '@', '<', '\n', '\r'])
        || selector.contains("/*")
    {
        return false;
    }
    // Pseudo-classes are checked outside quoted attribute values.
    let mut outside = String::with_capacity(selector.len());
    let mut quote = None;
    for c in selector.chars() {
        match (quote, c) {
            (None, '"' | '\'') => quote = Some(c),
            (Some(q), c) if c == q => quote = None,
            (None, c) => outside.push(c),
            _ => {}
        }
    }
    if quote.is_some() {
        return false;
    }
    let mut rest = outside.as_str();
    while let Some(at) = rest.find(':') {
        let after = rest[at + 1..].trim_start_matches(':');
        let name: String = after
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        if !PSEUDO.contains(&name.as_str()) {
            return false;
        }
        rest = &after[name.len()..];
    }
    true
}

/// Generic hiding rules (`##.ad-banner`, `###sponsored`) of the lists, minus the generic exceptions
/// (`#@#.ad-banner`). Site-specific rules are not here: the engine returns those per address.
pub fn generic_selectors(lists: &[String]) -> Vec<String> {
    let mut hide = Vec::new();
    let mut allowed = HashSet::new();
    for list in lists {
        for line in list.lines() {
            if let Some(selector) = line.strip_prefix("#@#") {
                allowed.insert(selector.trim().to_owned());
            } else if let Some(selector) = line.strip_prefix("##") {
                // `##+js(...)` is a scriptlet and `##^...` an HTML filter: neither is a selector.
                if !selector.starts_with(['+', '^']) && is_plain_selector(selector) {
                    hide.push(selector.trim().to_owned());
                }
            }
        }
    }
    let mut seen = HashSet::new();
    hide.retain(|selector| !allowed.contains(selector) && seen.insert(selector.clone()));
    hide
}

/// The style sheet that hides `selectors`, in groups.
pub fn stylesheet(selectors: &[String]) -> String {
    let mut css = String::new();
    for group in selectors.chunks(GROUP) {
        css.push_str(":is(");
        css.push_str(&group.join(","));
        css.push_str("){display:none!important}\n");
    }
    css
}

/// The generic hiding rules with their style sheet, made once per list update instead of once per
/// page: there are thousands of them and they are the same on every site that does not except any.
pub struct GenericHide {
    selectors: Vec<String>,
    /// The style sheet as a JavaScript string literal, ready for the injection script.
    json: String,
}

impl GenericHide {
    pub fn new(selectors: Vec<String>) -> Self {
        let json = json_text(&stylesheet(&selectors));
        Self { selectors, json }
    }
}

/// The hiding rules of one page: its own, and the generic ones that apply there.
pub struct PageCosmetics {
    site: String,
    generic: Generic,
}

enum Generic {
    /// The site turns generic hiding off (`$generichide`).
    Off,
    /// No generic rule is excepted here, so the prepared style sheet is used as is.
    Shared(Arc<GenericHide>),
    /// The site excepts some generic rules, so they are left out of a style sheet made for it.
    Filtered(String),
}

impl PageCosmetics {
    pub fn new(
        site: &[String],
        generic: Option<Arc<GenericHide>>,
        exceptions: &HashSet<String>,
    ) -> Self {
        let generic = match generic {
            None => Generic::Off,
            Some(shared) if !shared.selectors.iter().any(|s| exceptions.contains(s)) => {
                Generic::Shared(shared)
            }
            Some(shared) => {
                let kept: Vec<String> = shared
                    .selectors
                    .iter()
                    .filter(|s| !exceptions.contains(*s))
                    .cloned()
                    .collect();
                Generic::Filtered(stylesheet(&kept))
            }
        };
        Self {
            site: stylesheet(site),
            generic,
        }
    }

    /// The whole style sheet. The page itself gets it through [`PageCosmetics::script`].
    #[cfg(test)]
    pub fn css(&self) -> String {
        let generic = match &self.generic {
            Generic::Off => String::new(),
            Generic::Shared(shared) => stylesheet(&shared.selectors),
            Generic::Filtered(css) => css.clone(),
        };
        format!("{}{generic}", self.site)
    }

    /// The script that adds the style sheet to the page, or `None` when there is nothing to hide.
    pub fn script(&self) -> Option<String> {
        let generic = match &self.generic {
            Generic::Off => None,
            Generic::Shared(shared) if shared.selectors.is_empty() => None,
            Generic::Shared(shared) => Some(shared.json.clone()),
            Generic::Filtered(css) if css.is_empty() => None,
            Generic::Filtered(css) => Some(json_text(css)),
        };
        let site = (!self.site.is_empty()).then(|| json_text(&self.site));
        let expression = match (site, generic) {
            (None, None) => return None,
            (Some(site), None) => site,
            (None, Some(generic)) => generic,
            (Some(site), Some(generic)) => format!("{site}+{generic}"),
        };
        Some(injection_expression(&expression))
    }
}

fn json_text(text: &str) -> String {
    serde_json::to_string(text).expect("text is serializable")
}

/// A script that adds `css` to the page.
#[cfg(test)]
pub fn injection_script(css: &str) -> String {
    injection_expression(&json_text(css))
}

/// A script that adds the style sheet that the JavaScript expression `css` evaluates to. A
/// constructed style sheet is used first because a page's content security policy can refuse an
/// inline `<style>` but not this.
fn injection_expression(css: &str) -> String {
    format!(
        "(()=>{{try{{const css={css};const key='__sparkShieldCosmetic';\
         if(window[key])return;window[key]=true;\
         try{{const sheet=new CSSStyleSheet();sheet.replaceSync(css);\
         document.adoptedStyleSheets=[...document.adoptedStyleSheets,sheet];}}\
         catch(e){{const style=document.createElement('style');style.textContent=css;\
         (document.head||document.documentElement).appendChild(style);}}}}catch(e){{}}}})();"
    )
}

/// Hides the ads of the page that just loaded in `view`, when the shield is on for its site.
pub fn loaded(view: &Webview, state: &crate::state::browser_state::BrowserState, url: &str) {
    let on = state
        .policies()
        .shield_enabled()
        .lock()
        .map(|enabled| *enabled)
        .unwrap_or(false);
    let web = url::Url::parse(url).is_ok_and(|parsed| matches!(parsed.scheme(), "http" | "https"));
    if !on || !web || !super::adblock_engine::site_is_protected(state, url) {
        return;
    }
    let Some(lists) = view
        .app_handle()
        .try_state::<Arc<super::filter_lists::FilterLists>>()
    else {
        return;
    };
    if let Some(script) = lists.cosmetics(url).script() {
        crate::error_log::ignored(view.eval(script));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_plain_selectors_that_cannot_escape_a_rule_are_accepted() {
        for good in [
            ".ad-banner",
            "#sponsored",
            "div[class^=\"ad-\"]",
            "a[href*=\"//x.test/:y\"]",
            ".a:not(.b)",
            "div:has(> .ad)",
        ] {
            assert!(is_plain_selector(good), "{good}");
        }
        for bad in [
            "",
            ".a{color:red}",
            ".a;b",
            "x:has-text(Ad)",
            "x:xpath(//a)",
            ":-abp-has(a)",
            ".a\\:b",
            "a /* x",
            "@import",
            "x:style(a)",
            "a[b=\"c",
        ] {
            assert!(!is_plain_selector(bad), "{bad}");
        }
    }

    #[test]
    fn generic_rules_skip_scriptlets_procedural_rules_and_excepted_selectors() {
        let list = "[Adblock Plus 2.0]\n##.ad\n###promo\n##.ad\nsite.test##.only-here\n##+js(set-constant, a, b)\n##^script:has-text(x)\n##div:has-text(Ad)\n##.kept\n#@#.kept\n".to_owned();
        assert_eq!(generic_selectors(&[list]), [".ad", "#promo"]);
    }

    #[test]
    fn selectors_are_grouped_and_the_script_carries_the_css_as_text() {
        let selectors: Vec<String> = (0..250).map(|n| format!(".a{n}")).collect();
        let css = stylesheet(&selectors);
        assert_eq!(css.lines().count(), 3);
        assert!(css.starts_with(":is(.a0,.a1"));
        let script = injection_script("a{color:\"x\"}");
        assert!(script.contains("a{color:\\\"x\\\"}"));
    }

    #[test]
    fn the_prepared_generic_sheet_is_reused_unless_the_site_excepts_one_of_its_rules() {
        let generic = Arc::new(GenericHide::new(vec![".ad".into(), ".promo".into()]));
        let none = HashSet::new();
        let shared = PageCosmetics::new(&[".site".into()], Some(generic.clone()), &none);
        assert!(matches!(shared.generic, Generic::Shared(_)));
        assert_eq!(
            shared.css(),
            stylesheet(&[".site".into()]) + &stylesheet(&[".ad".into(), ".promo".into()])
        );
        let script = shared.script().unwrap();
        assert!(script.contains(".site") && script.contains(&generic.json));

        let excepted = HashSet::from([".ad".to_string()]);
        let filtered = PageCosmetics::new(&[], Some(generic.clone()), &excepted);
        assert!(!filtered.css().contains(".ad,") && filtered.css().contains(".promo"));

        assert!(PageCosmetics::new(&[], None, &none).script().is_none());
        let empty = Arc::new(GenericHide::new(Vec::new()));
        assert!(PageCosmetics::new(&[], Some(empty), &none)
            .script()
            .is_none());
    }
}
