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

/// A script that adds `css` to the page. A constructed style sheet is used first because a page's
/// content security policy can refuse an inline `<style>` but not this.
pub fn injection_script(css: &str) -> String {
    let css = serde_json::to_string(css).expect("text is serializable");
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
    let css = lists.cosmetic_css(url);
    if !css.is_empty() {
        crate::error_log::ignored(view.eval(injection_script(&css)));
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
}
