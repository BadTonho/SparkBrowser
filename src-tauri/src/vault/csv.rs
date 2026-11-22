//! CSV import and export of vault entries, in the layout Chrome, Edge and most managers use
//! (`name,url,username,password,note`).
//!
//! The exported file holds every password in plain text: callers must confirm the person first and
//! warn them. Nothing here logs; errors are stable codes that never contain file contents.
use super::model::EntryData;
use zeroize::Zeroizing;

/// A larger file is not a password export; it also bounds memory use.
pub const MAX_IMPORT_BYTES: u64 = 8 * 1024 * 1024;

/// What an import read from a file, before anything is stored.
#[derive(Debug, PartialEq)]
pub struct Parsed {
    pub entries: Vec<EntryData>,
    /// Rows that could not become an entry (no password, too long, no title and no address).
    pub skipped: usize,
}

/// Splits RFC 4180 text into rows. Quoted fields may hold commas, quotes (doubled) and line breaks.
fn rows(text: &str) -> Result<Vec<Vec<String>>, String> {
    let mut rows = Vec::new();
    let (mut row, mut field) = (Vec::new(), String::new());
    let (mut quoted, mut was_quoted) = (false, false);
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if quoted {
            match c {
                '"' if chars.peek() == Some(&'"') => {
                    field.push('"');
                    chars.next();
                }
                '"' => quoted = false,
                _ => field.push(c),
            }
            continue;
        }
        match c {
            '"' if field.is_empty() && !was_quoted => {
                quoted = true;
                was_quoted = true;
            }
            ',' => {
                row.push(std::mem::take(&mut field));
                was_quoted = false;
            }
            '\r' => {}
            '\n' => {
                row.push(std::mem::take(&mut field));
                was_quoted = false;
                rows.push(std::mem::take(&mut row));
            }
            _ => field.push(c),
        }
    }
    if quoted {
        return Err("vault.import_invalid".into());
    }
    if !field.is_empty() || !row.is_empty() || was_quoted {
        row.push(field);
        rows.push(row);
    }
    // Blank lines carry nothing.
    rows.retain(|row| !(row.len() == 1 && row[0].trim().is_empty()));
    Ok(rows)
}

#[derive(Default)]
struct Columns {
    title: Option<usize>,
    url: Option<usize>,
    username: Option<usize>,
    password: Option<usize>,
    notes: Option<usize>,
}

fn columns(header: &[String]) -> Columns {
    let mut columns = Columns::default();
    for (index, name) in header.iter().enumerate() {
        let slot = match name
            .trim()
            .trim_start_matches('\u{feff}')
            .to_lowercase()
            .as_str()
        {
            "name" | "title" => &mut columns.title,
            "url" | "website" | "web site" | "login_uri" | "origin" => &mut columns.url,
            "username" | "user" | "login" | "login_username" | "user name" => &mut columns.username,
            "password" | "login_password" => &mut columns.password,
            "note" | "notes" | "extra" | "comment" | "comments" => &mut columns.notes,
            _ => continue,
        };
        slot.get_or_insert(index);
    }
    columns
}

/// The host of an address, for naming an entry that came without a title.
fn host(url: &str) -> Option<String> {
    let rest = url.split("://").nth(1).unwrap_or(url);
    let host = rest.split(['/', '?', '#']).next()?.rsplit('@').next()?;
    let host = host.split(':').next()?.trim().trim_start_matches("www.");
    (!host.is_empty()).then(|| host.to_string())
}

pub fn parse(bytes: &[u8]) -> Result<Parsed, String> {
    let text = Zeroizing::new(
        String::from_utf8(bytes.to_vec()).map_err(|_| "vault.import_invalid".to_string())?,
    );
    let rows = rows(&text)?;
    let mut rows = rows.into_iter();
    let header = rows.next().ok_or("vault.import_invalid")?;
    let columns = columns(&header);
    let Some(password) = columns.password else {
        return Err("vault.import_invalid".into());
    };
    let cell = |row: &[String], index: Option<usize>| {
        index
            .and_then(|i| row.get(i))
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    };
    let (mut entries, mut skipped) = (Vec::new(), 0);
    for row in rows {
        let secret = row.get(password).cloned().unwrap_or_default();
        let url = cell(&row, columns.url);
        let username = cell(&row, columns.username);
        let mut title = cell(&row, columns.title);
        if title.is_empty() {
            title = host(&url).unwrap_or_else(|| username.clone());
        }
        let entry = EntryData {
            title,
            url,
            username,
            password: secret,
            // Notes keep their line breaks; only the ends are trimmed.
            notes: columns
                .notes
                .and_then(|i| row.get(i))
                .map(|s| s.trim().to_string())
                .unwrap_or_default(),
            totp: String::new(),
        };
        if entry.password.is_empty() || entry.validate().is_err() {
            skipped += 1;
        } else {
            entries.push(entry);
        }
    }
    Ok(Parsed { entries, skipped })
}

fn cell(text: &str) -> String {
    if text.contains([',', '"', '\n', '\r']) || text.starts_with(' ') || text.ends_with(' ') {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text.to_string()
    }
}

/// The file's bytes, wiped from memory when dropped. Same layout Chrome exports.
pub fn render(entries: &[EntryData]) -> Zeroizing<Vec<u8>> {
    let mut out = Zeroizing::new(String::from("name,url,username,password,note\r\n"));
    for entry in entries {
        let line = [
            &entry.title,
            &entry.url,
            &entry.username,
            &entry.password,
            &entry.notes,
        ]
        .iter()
        .map(|field| cell(field))
        .collect::<Vec<_>>()
        .join(",");
        out.push_str(&line);
        out.push_str("\r\n");
    }
    Zeroizing::new(out.as_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(title: &str, url: &str, user: &str, pw: &str, notes: &str) -> EntryData {
        EntryData {
            title: title.into(),
            url: url.into(),
            username: user.into(),
            password: pw.into(),
            notes: notes.into(),
            totp: String::new(),
        }
    }

    #[test]
    fn a_chrome_file_becomes_entries() {
        let text = "name,url,username,password,note\r\ndiscord.com,https://discord.com/login,me@x.test,hunter2,\r\n,https://www.roblox.com/,kid,pw2,a note\r\n";
        let parsed = parse(text.as_bytes()).unwrap();
        assert_eq!(parsed.skipped, 0);
        assert_eq!(
            parsed.entries[0],
            entry(
                "discord.com",
                "https://discord.com/login",
                "me@x.test",
                "hunter2",
                ""
            )
        );
        // A missing title is taken from the address.
        assert_eq!(parsed.entries[1].title, "roblox.com");
        assert_eq!(parsed.entries[1].notes, "a note");
    }

    #[test]
    fn quotes_commas_line_breaks_and_a_bom_are_handled() {
        let text = "\u{feff}name,url,username,password,note\n\"a, \"\"b\"\"\",https://x.test,u,\"p,w\",\"line1\nline2\"\n";
        let parsed = parse(text.as_bytes()).unwrap();
        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(parsed.entries[0].title, "a, \"b\"");
        assert_eq!(parsed.entries[0].password, "p,w");
        assert_eq!(parsed.entries[0].notes, "line1\nline2");
    }

    #[test]
    fn other_managers_column_names_work_and_unusable_rows_are_counted() {
        let text = "title,login_uri,login_username,login_password,notes,folder\nOne,https://one.test,u,p,,x\nTwo,https://two.test,u,,,x\n,,,p,,x\n";
        let parsed = parse(text.as_bytes()).unwrap();
        assert_eq!(parsed.entries.len(), 1);
        // No password, and no title or address at all.
        assert_eq!(parsed.skipped, 2);
    }

    #[test]
    fn a_file_that_is_not_a_password_export_is_refused() {
        for bad in ["", "a,b,c\n1,2,3\n", "name,url\nx,y\n"] {
            assert_eq!(
                parse(bad.as_bytes()).unwrap_err(),
                "vault.import_invalid",
                "{bad:?}"
            );
        }
        assert_eq!(
            parse(b"name,password\n\"unterminated,x\n").unwrap_err(),
            "vault.import_invalid"
        );
        assert_eq!(
            parse(&[0xff, 0xfe, 0x00]).unwrap_err(),
            "vault.import_invalid"
        );
    }

    #[test]
    fn an_export_reads_back_the_same() {
        let entries = vec![
            entry(
                "Bank",
                "https://bank.test",
                "me",
                "pa,ss\"word",
                "multi\nline",
            ),
            entry("Plain", "", "", "x", ""),
            entry(" spaced ", "https://a.test", "u", "p", ""),
        ];
        let bytes = render(&entries);
        assert!(String::from_utf8_lossy(&bytes).starts_with("name,url,username,password,note\r\n"));
        let parsed = parse(&bytes).unwrap();
        assert_eq!(parsed.skipped, 0);
        assert_eq!(parsed.entries.len(), 3);
        assert_eq!(parsed.entries[0], entries[0]);
        assert_eq!(parsed.entries[1].password, "x");
        // Spaces around a title are not kept on import.
        assert_eq!(parsed.entries[2].title, "spaced");
    }
}
