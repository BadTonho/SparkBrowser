use super::http;
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    pub sender: String,
    pub subject: String,
    pub date: u64,
    pub unread: bool,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Inbox {
    pub email: String,
    pub unread: u64,
    pub messages: Vec<Message>,
    pub updated: u64,
}
pub fn profile(base: &str, token: &str) -> Result<String, String> {
    let value: Value = http::json(http::send(
        http::client()?
            .get(format!("{base}/users/me/profile"))
            .bearer_auth(token)
            .query(&[("fields", "emailAddress")]),
    )?)?;
    value["emailAddress"]
        .as_str()
        .filter(|s| s.len() < 320 && s.contains('@'))
        .map(str::to_owned)
        .ok_or_else(|| "invalid_response".into())
}
pub fn inbox(base: &str, token: &str, email: &str) -> Result<Inbox, String> {
    let client = http::client()?;
    let label: Value = http::json(http::send(
        client
            .get(format!("{base}/users/me/labels/INBOX"))
            .bearer_auth(token)
            .query(&[("fields", "messagesUnread")]),
    )?)?;
    let list: Value = http::json(http::send(
        client
            .get(format!("{base}/users/me/messages"))
            .bearer_auth(token)
            .query(&[
                ("labelIds", "INBOX"),
                ("maxResults", "5"),
                ("fields", "messages(id)"),
            ]),
    )?)?;
    let mut messages = vec![];
    if let Some(rows) = list["messages"].as_array() {
        for row in rows.iter().take(5) {
            let id = row["id"]
                .as_str()
                .filter(|s| {
                    !s.is_empty() && s.len() <= 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
                })
                .ok_or("invalid_response")?;
            let message: Value = http::json(http::send(
                client
                    .get(format!("{base}/users/me/messages/{id}"))
                    .bearer_auth(token)
                    .query(&[
                        ("format", "metadata"),
                        ("metadataHeaders", "From"),
                        ("metadataHeaders", "Subject"),
                        ("fields", "internalDate,labelIds,payload/headers"),
                    ]),
            )?)?;
            messages.push(parse_message(&message)?);
        }
    }
    Ok(Inbox {
        email: email.into(),
        unread: label["messagesUnread"].as_u64().ok_or("invalid_response")?,
        messages,
        updated: super::now(),
    })
}
fn parse_message(v: &Value) -> Result<Message, String> {
    let headers = v["payload"]["headers"]
        .as_array()
        .ok_or("invalid_response")?;
    let header = |name: &str| {
        headers
            .iter()
            .find(|h| {
                h["name"]
                    .as_str()
                    .is_some_and(|n| n.eq_ignore_ascii_case(name))
            })
            .and_then(|h| h["value"].as_str())
            .unwrap_or("")
            .chars()
            .filter(|c| !c.is_control())
            .take(500)
            .collect::<String>()
    };
    Ok(Message {
        sender: header("From"),
        subject: header("Subject"),
        date: v["internalDate"]
            .as_str()
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|v| *v <= 253402300799000)
            .ok_or("invalid_response")?
            / 1000,
        unread: v["labelIds"]
            .as_array()
            .is_some_and(|labels| labels.iter().any(|l| l == "UNREAD")),
    })
}
pub fn fetch_feed(feed_url: &str, cookies: &str) -> Result<Inbox, String> {
    let client = http::client()?;
    let res = client
        .get(feed_url)
        .header(reqwest::header::COOKIE, cookies)
        .header(
            reqwest::header::USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
        )
        .header(reqwest::header::ACCEPT, "text/xml, application/xml, */*")
        .send()
        .map_err(|e| e.to_string())?;

    if res.status() == reqwest::StatusCode::UNAUTHORIZED
        || res.status() == reqwest::StatusCode::FORBIDDEN
    {
        return Err("not_authenticated".into());
    }
    if !res.status().is_success() {
        return Err("feed_unavailable".into());
    }
    let body = res.text().map_err(|e| e.to_string())?;
    parse_feed_xml(&body)
}

fn extract_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)?;
    let after = &xml[start + open.len()..];
    let end = after.find(&close)?;
    Some(after[..end].trim().to_string())
}

fn decode_xml(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .chars()
        .filter(|c| !c.is_control())
        .take(500)
        .collect()
}

fn is_leap_year(year: u64) -> bool {
    year.is_multiple_of(4) && !year.is_multiple_of(100) || year.is_multiple_of(400)
}

pub fn parse_iso_date(raw: &str) -> Option<u64> {
    let parts: Vec<&str> = raw.split('T').collect();
    if parts.len() != 2 {
        return None;
    }
    let ymd: Vec<u64> = parts[0].split('-').filter_map(|s| s.parse().ok()).collect();
    if ymd.len() != 3 {
        return None;
    }
    let (year, month, day) = (ymd[0], ymd[1], ymd[2]);
    let time_str = parts[1].trim_end_matches('Z');
    let hms: Vec<u64> = time_str
        .split(':')
        .filter_map(|s| s.split('+').next()?.split('-').next()?.parse().ok())
        .collect();
    if hms.len() < 2 {
        return None;
    }
    let hour = hms[0];
    let min = hms[1];
    let sec = hms.get(2).copied().unwrap_or(0);

    let mut days: u64 = 0;
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }
    let month_days = [
        31,
        if is_leap_year(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    for m in 1..month {
        days += month_days[(m - 1) as usize];
    }
    days += day.saturating_sub(1);
    Some(days * 86400 + hour * 3600 + min * 60 + sec)
}

pub fn parse_feed_xml(xml: &str) -> Result<Inbox, String> {
    if !xml.contains("<feed") {
        return Err("invalid_response".into());
    }

    let unread = extract_tag(xml, "fullcount")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    let email = extract_tag(xml, "title")
        .and_then(|t| {
            t.split_whitespace()
                .find(|w| w.contains('@') && w.contains('.'))
                .map(|e| {
                    e.trim_matches(|c: char| {
                        !c.is_alphanumeric() && c != '@' && c != '.' && c != '_' && c != '-'
                    })
                    .to_string()
                })
        })
        .unwrap_or_default();

    let mut messages = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<entry>") {
        let after_start = &rest[start + 7..];
        let Some(end) = after_start.find("</entry>") else {
            break;
        };
        let entry = &after_start[..end];
        rest = &after_start[end + 8..];

        let subject = extract_tag(entry, "title")
            .map(|s| decode_xml(&s))
            .unwrap_or_default();
        let sender = extract_tag(entry, "name")
            .or_else(|| extract_tag(entry, "email"))
            .map(|s| decode_xml(&s))
            .unwrap_or_default();
        let date_str = extract_tag(entry, "issued")
            .or_else(|| extract_tag(entry, "modified"))
            .unwrap_or_default();
        let date = parse_iso_date(&date_str).unwrap_or_else(super::now);

        messages.push(Message {
            sender,
            subject,
            date,
            unread: true,
        });
        if messages.len() >= 5 {
            break;
        }
    }

    Ok(Inbox {
        email,
        unread,
        messages,
        updated: super::now(),
    })
}

pub fn inbox_url(email: &str) -> url::Url {
    let mut url = url::Url::parse("https://mail.google.com/mail/u/").unwrap();
    url.query_pairs_mut().append_pair("authuser", email);
    url.set_fragment(Some("inbox"));
    url
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_metadata_is_presented() {
        let message=parse_message(&serde_json::json!({"internalDate":"100000","labelIds":["INBOX","UNREAD"],"payload":{"headers":[{"name":"FROM","value":"<script>Sender</script>\r\n"},{"name":"Subject","value":"Hello"}],"body":{"data":"must-not-leak"}},"snippet":"must-not-leak"})).unwrap();
        assert!(message.unread);
        assert_eq!(message.sender, "<script>Sender</script>");
        assert!(!serde_json::to_string(&message)
            .unwrap()
            .contains("must-not-leak"));
        assert_eq!(
            inbox_url("a+b@example.com").query_pairs().next().unwrap().1,
            "a+b@example.com"
        );
    }
    #[test]
    fn feed_xml_parsing_works() {
        let sample_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed version="0.3" xmlns="http://purl.org/atom/ns#">
  <title>Gmail - Inbox for user@gmail.com</title>
  <fullcount>4</fullcount>
  <entry>
    <title>&lt;b&gt;Subject&lt;/b&gt;</title>
    <author>
      <name>&lt;Sender&gt;</name>
      <email>sender@gmail.com</email>
    </author>
    <issued>2026-11-04T12:00:00Z</issued>
  </entry>
</feed>"#;
        let inbox = parse_feed_xml(sample_xml).unwrap();
        assert_eq!(inbox.email, "user@gmail.com");
        assert_eq!(inbox.unread, 4);
        assert_eq!(inbox.messages.len(), 1);
        assert_eq!(inbox.messages[0].subject, "<b>Subject</b>");
        assert_eq!(inbox.messages[0].sender, "<Sender>");
        assert!(inbox.messages[0].date > 0);
    }
}
