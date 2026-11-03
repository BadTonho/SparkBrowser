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
}
