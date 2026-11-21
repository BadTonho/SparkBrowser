//! What the browser sends to the page script: the list to show, and the values to fill in.
use crate::vault::model::EntryData;
use crate::vault::records::{Card, Contact};
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct Item {
    pub id: String,
    pub title: String,
    pub subtitle: String,
}

pub fn locked(hello: bool) -> Value {
    json!({ "op": "show", "state": "locked", "hello": hello })
}

pub fn items(kind: &str, items: Vec<Item>) -> Value {
    let rows: Vec<Value> = items
        .into_iter()
        .map(|item| json!({ "id": item.id, "title": item.title, "subtitle": item.subtitle }))
        .collect();
    json!({ "op": "show", "state": "items", "kind": kind, "items": rows })
}

pub fn fill(kind: &str, values: HashMap<&'static str, String>) -> Value {
    json!({ "op": "fill", "kind": kind, "values": values })
}

pub fn login_values(entry: &EntryData) -> HashMap<&'static str, String> {
    HashMap::from([
        ("username", entry.username.clone()),
        ("password", entry.password.clone()),
    ])
}

pub fn card_values(card: &Card) -> HashMap<&'static str, String> {
    HashMap::from([
        ("cc-number", card.number.clone()),
        ("cc-name", card.holder.clone()),
        ("cc-exp-month", format!("{:02}", card.exp_month)),
        ("cc-exp-year", card.exp_year.to_string()),
    ])
}

pub fn contact_values(contact: &Contact) -> HashMap<&'static str, String> {
    let (given, family) = match contact.name.trim().split_once(' ') {
        Some((first, rest)) => (first.to_string(), rest.trim().to_string()),
        None => (contact.name.trim().to_string(), String::new()),
    };
    let street_address = [contact.street.trim(), contact.street2.trim()]
        .iter()
        .filter(|part| !part.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(", ");
    let pairs = [
        ("name", contact.name.clone()),
        ("given-name", given),
        ("family-name", family),
        ("organization", contact.organization.clone()),
        ("email", contact.email.clone()),
        ("tel", contact.phone.clone()),
        ("street-address", street_address),
        ("address-line1", contact.street.clone()),
        ("address-line2", contact.street2.clone()),
        ("address-level2", contact.city.clone()),
        ("address-level1", contact.region.clone()),
        ("postal-code", contact.postal_code.clone()),
        ("country-name", contact.country.clone()),
    ];
    pairs
        .into_iter()
        .filter(|(_, value)| !value.trim().is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_card_is_split_into_what_the_fields_ask_for_and_never_carries_a_security_code() {
        let card = Card {
            nickname: "Main".into(),
            holder: "Ana Souza".into(),
            number: "4242424242424242".into(),
            exp_month: 7,
            exp_year: 2031,
        };
        let values = card_values(&card);
        assert_eq!(values["cc-exp-month"], "07");
        assert_eq!(values["cc-exp-year"], "2031");
        assert!(!values.contains_key("cc-csc"));
    }

    #[test]
    fn a_contact_name_is_split_and_empty_fields_are_left_out() {
        let contact = Contact {
            name: "Ana Maria Souza".into(),
            street: "Rua A, 10".into(),
            ..Contact::default()
        };
        let values = contact_values(&contact);
        assert_eq!(values["given-name"], "Ana");
        assert_eq!(values["family-name"], "Maria Souza");
        assert_eq!(values["street-address"], "Rua A, 10");
        assert!(!values.contains_key("email"));
    }

    #[test]
    fn the_list_message_carries_only_titles() {
        let message = items(
            "card",
            vec![Item {
                id: "id1".into(),
                title: "Visa •••• 4242".into(),
                subtitle: "Ana 07/31".into(),
            }],
        );
        let text = message.to_string();
        assert!(text.contains("4242") && !text.contains("4242424242424242"));
    }
}
