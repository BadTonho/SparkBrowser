//! Bounded RSS/Atom text projection; executable markup never reaches the UI.
use super::Entry;
use quick_xml::{
    events::{BytesStart, Event},
    Reader,
};

fn atom_link(element: &BytesStart<'_>, base: &url::Url) -> Result<Option<String>, String> {
    let mut href = None;
    let mut rel = String::new();
    let mut kind = String::new();
    for attr in element.attributes() {
        let attr = attr.map_err(|_| "feeds.invalid_xml")?;
        let value = attr
            .normalized_value(quick_xml::XmlVersion::Implicit1_0)
            .map_err(|_| "feeds.invalid_xml")?
            .into_owned();
        match attr.key.as_ref() {
            "href" => href = Some(value),
            "rel" => rel = value,
            "type" => kind = value,
            _ => {}
        }
    }
    if !matches!(rel.as_str(), "" | "alternate")
        || !matches!(kind.as_str(), "" | "text/html" | "application/xhtml+xml")
    {
        return Ok(None);
    }
    Ok(href.and_then(|href| web_link(base, &href)))
}

fn web_link(base: &url::Url, raw: &str) -> Option<String> {
    if raw.trim().is_empty() {
        return None;
    }
    base.join(raw.trim())
        .ok()
        .filter(|url| {
            matches!(url.scheme(), "http" | "https")
                && url.host_str().is_some()
                && url.username().is_empty()
                && url.password().is_none()
        })
        .map(|url| url.to_string())
}

fn append(entry: &mut Entry, field: Option<&str>, text: &str) {
    match field {
        Some("title") => entry.title.push_str(text),
        Some("link") => entry.url.push_str(text),
        Some("description" | "summary" | "content") => entry.text.push_str(text),
        _ => {}
    }
}

pub(super) fn parse(xml: &str, base: &str) -> Result<Vec<Entry>, String> {
    let base = url::Url::parse(base).map_err(|_| "feeds.invalid_url")?;
    let mut reader = Reader::from_str(xml);
    let mut stack = Vec::new();
    let mut current: Option<(usize, Entry, Option<String>)> = None;
    let mut result = Vec::new();
    let mut format = false;
    loop {
        match reader.read_event().map_err(|_| "feeds.invalid_xml")? {
            Event::Start(e) => {
                let name = e.local_name().as_ref().to_ascii_lowercase();
                stack.push(name.clone());
                if stack.len() > 64 {
                    return Err("feeds.invalid_xml".into());
                }
                if stack.len() == 1 && matches!(name.as_str(), "rss" | "feed" | "rdf") {
                    format = true;
                }
                if current.is_none() && matches!(name.as_str(), "item" | "entry") {
                    current = Some((stack.len(), Entry::default(), None));
                } else if let Some((depth, _, link)) = &mut current {
                    if stack.len() == *depth + 1 && name == "link" && link.is_none() {
                        *link = atom_link(&e, &base)?;
                    }
                }
            }
            Event::Empty(e) => {
                let name = e.local_name().as_ref().to_ascii_lowercase();
                if stack.is_empty() && matches!(name.as_str(), "rss" | "feed" | "rdf") {
                    format = true;
                }
                if let Some((depth, _, link)) = &mut current {
                    if stack.len() == *depth && name == "link" && link.is_none() {
                        *link = atom_link(&e, &base)?;
                    }
                }
            }
            Event::Text(e) => {
                if let Some((depth, entry, _)) = &mut current {
                    let text =
                        quick_xml::escape::unescape(e.as_ref()).map_err(|_| "feeds.invalid_xml")?;
                    append(entry, stack.get(*depth).map(String::as_str), &text);
                }
            }
            Event::GeneralRef(e) => {
                if let Some((depth, entry, _)) = &mut current {
                    let encoded = format!("&{};", e.as_ref());
                    let text =
                        quick_xml::escape::unescape(&encoded).map_err(|_| "feeds.invalid_xml")?;
                    append(entry, stack.get(*depth).map(String::as_str), &text);
                }
            }
            Event::CData(e) => {
                if let Some((depth, entry, _)) = &mut current {
                    append(entry, stack.get(*depth).map(String::as_str), e.as_ref());
                }
            }
            Event::End(_) => {
                if current
                    .as_ref()
                    .is_some_and(|(depth, _, _)| *depth == stack.len())
                {
                    let (_, mut entry, preferred) = current.take().ok_or("feeds.invalid_xml")?;
                    if let Some(link) = preferred.or_else(|| web_link(&base, &entry.url)) {
                        entry.url = link;
                        entry.title = entry.title.trim().chars().take(500).collect();
                        entry.text = entry.text.trim().chars().take(20_000).collect();
                        if result.len() < 200 {
                            result.push(entry);
                        }
                    }
                }
                stack.pop().ok_or("feeds.invalid_xml")?;
            }
            Event::DocType(_) => return Err("feeds.unsupported_xml".into()),
            Event::Eof => {
                if !stack.is_empty() {
                    return Err("feeds.invalid_xml".into());
                }
                break;
            }
            _ => {}
        }
    }
    if !format {
        return Err("feeds.invalid_format".into());
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cdata_and_nested_text_survive_and_atom_links_point_to_articles() {
        let rss = parse("<rss><channel><item><title><![CDATA[News & views]]></title><link><![CDATA[/news]]></link><description>Read <b>this</b> now.</description></item></channel></rss>", "https://example.test/").unwrap();
        assert_eq!(rss[0].title, "News & views");
        assert_eq!(rss[0].text, "Read this now.");
        assert_eq!(rss[0].url, "https://example.test/news");
        for links in [
            "<link rel='self' href='/api'/><link rel='alternate' type='text/html' href='/article'/><link rel='enclosure' href='/audio'/>",
            "<link href='/article'></link><link rel='self' href='/api'/>",
            "<link type='application/atom+xml' href='/api'/><link href='/article'/>",
        ] {
            let atom = parse(&format!("<feed><entry><title>Title</title>{links}</entry></feed>"), "https://example.test/").unwrap();
            assert_eq!(atom[0].url, "https://example.test/article");
        }
    }
}
