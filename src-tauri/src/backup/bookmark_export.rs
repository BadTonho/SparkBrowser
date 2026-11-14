//! Bookmark export for other browsers: the Netscape bookmark file (read by Chrome, Edge, Firefox,
//! Safari and most managers) and a flat CSV. Pure formatting; the command owns the file dialog.
use crate::database::bookmarks_repo::{BookmarkData, BookmarkItem};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    Html,
    Csv,
}
impl Format {
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Format::Html => &["html", "htm"],
            Format::Csv => &["csv"],
        }
    }
    pub fn default_name(self) -> &'static str {
        match self {
            Format::Html => "bookmarks.html",
            Format::Csv => "bookmarks.csv",
        }
    }
}

pub struct Exported {
    pub bytes: Vec<u8>,
    pub count: usize,
    /// Bookmarks left out because their address is not plain http(s).
    pub skipped: usize,
}

/// A hostile `javascript:` or `data:` address must never travel into another browser's toolbar.
fn exportable(item: &BookmarkItem) -> bool {
    url::Url::parse(&item.url)
        .is_ok_and(|url| matches!(url.scheme(), "http" | "https") && url.host_str().is_some())
}

/// Bookmarks split for output: per folder in folder order, then the loose ones.
struct Grouped<'a> {
    /// `(folder name, items)`; folders without exportable bookmarks are left out.
    folders: Vec<(&'a str, Vec<&'a BookmarkItem>)>,
    loose: Vec<&'a BookmarkItem>,
    skipped: usize,
}

/// A dangling folder id counts as loose.
fn grouped(data: &BookmarkData) -> Grouped<'_> {
    let mut skipped = 0;
    let mut by_folder: HashMap<i64, Vec<&BookmarkItem>> = HashMap::new();
    let mut loose = Vec::new();
    let known: HashMap<i64, ()> = data.folders.iter().map(|f| (f.id, ())).collect();
    for item in &data.bookmarks {
        if !exportable(item) {
            skipped += 1;
            continue;
        }
        match item.folder_id.filter(|id| known.contains_key(id)) {
            Some(id) => by_folder.entry(id).or_default().push(item),
            None => loose.push(item),
        }
    }
    let folders = data
        .folders
        .iter()
        .filter_map(|folder| {
            by_folder
                .remove(&folder.id)
                .map(|items| (folder.name.as_str(), items))
        })
        .collect();
    Grouped {
        folders,
        loose,
        skipped,
    }
}

fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            // Control characters other than whitespace have no place in a bookmark title.
            c if c.is_control() && !matches!(c, '\t' | '\n' | '\r') => {}
            c => out.push(c),
        }
    }
    out
}

fn link(item: &BookmarkItem, indent: &str) -> String {
    let title = if item.title.trim().is_empty() {
        &item.url
    } else {
        &item.title
    };
    format!(
        "{indent}<DT><A HREF=\"{}\">{}</A>\n",
        escape_html(&item.url),
        escape_html(title)
    )
}

pub fn html(data: &BookmarkData) -> Exported {
    let Grouped {
        folders,
        loose,
        skipped,
    } = grouped(data);
    let mut out = String::from(
        "<!DOCTYPE NETSCAPE-Bookmark-file-1>\n\
         <!-- This is an automatically generated file. It will be read and overwritten. DO NOT EDIT! -->\n\
         <META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">\n\
         <TITLE>Bookmarks</TITLE>\n\
         <H1>Bookmarks</H1>\n\
         <DL><p>\n",
    );
    let mut count = 0;
    for (name, items) in &folders {
        out.push_str(&format!(
            "    <DT><H3>{}</H3>\n    <DL><p>\n",
            escape_html(name)
        ));
        for item in items {
            out.push_str(&link(item, "        "));
            count += 1;
        }
        out.push_str("    </DL><p>\n");
    }
    for item in &loose {
        out.push_str(&link(item, "    "));
        count += 1;
    }
    out.push_str("</DL><p>\n");
    Exported {
        bytes: out.into_bytes(),
        count,
        skipped,
    }
}

/// RFC 4180 quoting, plus a leading apostrophe on cells a spreadsheet could run as a formula
/// (page titles are controlled by websites).
fn csv_cell(text: &str) -> String {
    let guarded = if text.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{text}")
    } else {
        text.to_owned()
    };
    if guarded.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", guarded.replace('"', "\"\""))
    } else {
        guarded
    }
}

/// UTF-8 with a byte order mark so Excel reads accented titles correctly; CRLF line endings.
pub fn csv(data: &BookmarkData) -> Exported {
    let Grouped {
        folders,
        loose,
        skipped,
    } = grouped(data);
    let mut out = String::from("\u{feff}folder,title,url\r\n");
    let mut count = 0;
    let rows = folders
        .iter()
        .flat_map(|(name, items)| items.iter().map(move |item| (*name, *item)))
        .chain(loose.iter().map(|item| ("", *item)));
    for (folder, item) in rows {
        out.push_str(&format!(
            "{},{},{}\r\n",
            csv_cell(folder),
            csv_cell(&item.title),
            csv_cell(&item.url)
        ));
        count += 1;
    }
    Exported {
        bytes: out.into_bytes(),
        count,
        skipped,
    }
}

pub fn render(format: Format, data: &BookmarkData) -> Exported {
    match format {
        Format::Html => html(data),
        Format::Csv => csv(data),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::bookmarks_repo::BookmarkFolder;

    fn item(id: i64, title: &str, url: &str, folder: Option<i64>) -> BookmarkItem {
        BookmarkItem {
            id,
            title: title.into(),
            url: url.into(),
            icon: "x".into(),
            folder_id: folder,
        }
    }
    fn sample() -> BookmarkData {
        BookmarkData {
            folders: vec![
                BookmarkFolder {
                    id: 1,
                    name: "Work & <Play>".into(),
                },
                BookmarkFolder {
                    id: 2,
                    name: "Empty".into(),
                },
            ],
            bookmarks: vec![
                item(1, "Loose \"one\"", "https://example.test/a?x=1&y=2", None),
                item(2, "In folder", "https://example.test/b", Some(1)),
                item(3, "Dangling", "https://example.test/c", Some(99)),
                item(4, "Bad", "javascript:alert(1)", None),
                item(5, "Data", "data:text/html,hi", Some(1)),
            ],
        }
    }

    #[test]
    fn html_is_netscape_format_escaped_grouped_and_skips_unsafe_addresses() {
        let out = html(&sample());
        let text = String::from_utf8(out.bytes).unwrap();
        assert!(text.starts_with("<!DOCTYPE NETSCAPE-Bookmark-file-1>"));
        assert_eq!((out.count, out.skipped), (3, 2));
        assert!(text.contains("<DT><H3>Work &amp; &lt;Play&gt;</H3>"));
        assert!(
            text.contains("HREF=\"https://example.test/a?x=1&amp;y=2\">Loose &quot;one&quot;</A>")
        );
        assert!(!text.contains("javascript:") && !text.contains("data:text"));
        assert!(!text.contains("Empty"), "empty folders are not exported");
        // The folder's bookmark sits inside its list, loose ones after it.
        let folder = text.find("In folder").unwrap();
        assert!(folder < text.find("Loose").unwrap());
        assert!(folder > text.find("<H3>").unwrap());
        assert!(
            text.contains("Dangling"),
            "a dangling folder id falls back to the root"
        );
        assert_eq!(
            text.matches("<DL><p>").count(),
            text.matches("</DL><p>").count()
        );
    }

    #[test]
    fn html_uses_the_address_when_the_title_is_blank_and_drops_control_characters() {
        let data = BookmarkData {
            folders: vec![],
            bookmarks: vec![
                item(1, "  ", "https://example.test/blank", None),
                item(2, "Bell\u{7}ring", "https://example.test/bell", None),
            ],
        };
        let text = String::from_utf8(html(&data).bytes).unwrap();
        assert!(text.contains(">https://example.test/blank</A>"));
        assert!(text.contains(">Bellring</A>"));
    }

    #[test]
    fn csv_quotes_fields_guards_formulas_and_starts_with_a_bom() {
        let mut data = sample();
        data.bookmarks
            .push(item(6, "=HYPERLINK(\"x\")", "https://example.test/f", None));
        data.bookmarks
            .push(item(7, "a, b\nc", "https://example.test/g", Some(2)));
        let out = csv(&data);
        let text = String::from_utf8(out.bytes).unwrap();
        assert!(text.starts_with("\u{feff}folder,title,url\r\n"));
        assert_eq!((out.count, out.skipped), (5, 2));
        assert!(text.contains("Work & <Play>,In folder,https://example.test/b\r\n"));
        assert!(text.contains(",\"Loose \"\"one\"\"\",https://example.test/a?x=1&y=2\r\n"));
        assert!(text.contains(",\"'=HYPERLINK(\"\"x\"\")\",https://example.test/f\r\n"));
        assert!(text.contains("Empty,\"a, b\nc\",https://example.test/g\r\n"));
        assert!(!text.contains("javascript:"));
    }

    #[test]
    fn nothing_to_export_still_produces_valid_files() {
        let empty = BookmarkData {
            bookmarks: vec![],
            folders: vec![],
        };
        assert_eq!(html(&empty).count, 0);
        assert!(String::from_utf8(html(&empty).bytes)
            .unwrap()
            .contains("<DL><p>\n</DL><p>"));
        assert_eq!(csv(&empty).bytes, "\u{feff}folder,title,url\r\n".as_bytes());
    }
}
