//! Site permission choices for engines that keep none of their own (WebKitGTK).
use crate::site_permissions::{PermissionKind, PermissionState, SitePermission};
use rusqlite::{params, Connection, OptionalExtension};

fn kind_name(kind: PermissionKind) -> &'static str {
    match kind {
        PermissionKind::Camera => "camera",
        PermissionKind::Microphone => "microphone",
        PermissionKind::Location => "location",
        PermissionKind::Notifications => "notifications",
    }
}

fn kind_from(name: &str) -> Option<PermissionKind> {
    Some(match name {
        "camera" => PermissionKind::Camera,
        "microphone" => PermissionKind::Microphone,
        "location" => PermissionKind::Location,
        "notifications" => PermissionKind::Notifications,
        _ => return None,
    })
}

pub fn migrate(conn: &Connection) -> Result<(), String> {
    super::migrations::apply(conn, "site-permissions-v1", |c| {
        c.execute_batch(
            "CREATE TABLE site_permissions(origin TEXT NOT NULL,kind TEXT NOT NULL,\
             state TEXT NOT NULL CHECK(state IN('allow','block')),PRIMARY KEY(origin,kind))",
        )
        .map_err(|e| e.to_string())
    })
}

pub fn list(conn: &Connection) -> Result<Vec<SitePermission>, String> {
    let rows = conn
        .prepare("SELECT origin,kind,state FROM site_permissions ORDER BY origin,kind")
        .map_err(|e| e.to_string())?
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows
        .into_iter()
        .filter_map(|(origin, kind, state)| {
            Some(SitePermission {
                origin,
                kind: kind_from(&kind)?,
                state: if state == "allow" {
                    PermissionState::Allow
                } else {
                    PermissionState::Block
                },
            })
        })
        .collect())
}

/// The saved choice for one origin; `Ask` when the user has not chosen.
pub fn get(
    conn: &Connection,
    origin: &str,
    kind: PermissionKind,
) -> Result<PermissionState, String> {
    let state: Option<String> = conn
        .query_row(
            "SELECT state FROM site_permissions WHERE origin=?1 AND kind=?2",
            params![origin, kind_name(kind)],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    Ok(match state.as_deref() {
        Some("allow") => PermissionState::Allow,
        Some("block") => PermissionState::Block,
        _ => PermissionState::Ask,
    })
}

/// Saves a choice; `Ask` forgets it.
pub fn set(
    conn: &Connection,
    origin: &str,
    kind: PermissionKind,
    state: PermissionState,
) -> Result<(), String> {
    let name = kind_name(kind);
    match state {
        PermissionState::Ask => conn.execute(
            "DELETE FROM site_permissions WHERE origin=?1 AND kind=?2",
            params![origin, name],
        ),
        PermissionState::Allow | PermissionState::Block => conn.execute(
            "INSERT INTO site_permissions(origin,kind,state) VALUES(?1,?2,?3) \
             ON CONFLICT(origin,kind) DO UPDATE SET state=excluded.state",
            params![
                origin,
                name,
                if state == PermissionState::Allow {
                    "allow"
                } else {
                    "block"
                }
            ],
        ),
    }
    .map(|_| ())
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn choices_are_saved_changed_and_forgotten() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let site = "https://example.test";
        assert_eq!(
            get(&conn, site, PermissionKind::Camera).unwrap(),
            PermissionState::Ask
        );
        set(&conn, site, PermissionKind::Camera, PermissionState::Allow).unwrap();
        set(&conn, site, PermissionKind::Camera, PermissionState::Block).unwrap();
        assert_eq!(
            get(&conn, site, PermissionKind::Camera).unwrap(),
            PermissionState::Block
        );
        assert_eq!(list(&conn).unwrap().len(), 1);
        set(&conn, site, PermissionKind::Camera, PermissionState::Ask).unwrap();
        assert!(list(&conn).unwrap().is_empty());
    }
}
