use crate::state::workspace_model::WorkspaceItem;
use rusqlite::{params, Connection, OptionalExtension};

const LEGACY_DEFAULT_WORKSPACE_COLORS: [(&str, &str, &str); 4] = [
    ("ws-personal", "#6e8efb", "#a777e3"),
    ("ws-work", "#38bdf8", "#6366f1"),
    ("ws-study", "#34d399", "#059669"),
    ("ws-gaming", "#f43f5e", "#d946ef"),
];

pub fn initialize_workspace_table(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS workspaces (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            name_key TEXT NOT NULL UNIQUE,
            icon TEXT NOT NULL,
            accent_primary TEXT NOT NULL,
            accent_secondary TEXT NOT NULL,
            position INTEGER NOT NULL UNIQUE
        );",
    )
    .map_err(|e| format!("Failed to create workspaces table: {e}"))?;

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM workspaces", [], |row| row.get(0))
        .map_err(|e| format!("Failed to inspect workspaces: {e}"))?;
    if count == 0 {
        for (position, workspace) in WorkspaceItem::default_workspaces().iter().enumerate() {
            conn.execute(
                "INSERT INTO workspaces (id, name, name_key, icon, accent_primary, accent_secondary, position)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    workspace.id,
                    workspace.name,
                    workspace.name.to_lowercase(),
                    workspace.icon,
                    workspace.accent_primary,
                    workspace.accent_secondary,
                    position as i64
                ],
            )
            .map_err(|e| format!("Failed to seed default workspaces: {e}"))?;
        }
    }

    migrate_legacy_default_workspace_colors(conn)?;

    Ok(())
}

fn migrate_legacy_default_workspace_colors(conn: &Connection) -> Result<(), String> {
    crate::database::migrations::apply(conn, "workspace-palette-v1", |tx| {
        // Older releases used user_version for this migration. Import its marker without reapplying it.
        let legacy_version: i64 = tx
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(|e| e.to_string())?;
        if legacy_version >= 1 {
            return Ok(());
        }
        let defaults = WorkspaceItem::default_workspaces();
        for (id, primary, secondary) in LEGACY_DEFAULT_WORKSPACE_COLORS {
            if let Some(default) = defaults.iter().find(|item| item.id == id) {
                tx.execute("UPDATE workspaces SET accent_primary = ?1 WHERE id = ?2 AND lower(accent_primary) = ?3", params![default.accent_primary, id, primary]).map_err(|e| e.to_string())?;
                tx.execute("UPDATE workspaces SET accent_secondary = ?1 WHERE id = ?2 AND lower(accent_secondary) = ?3", params![default.accent_secondary, id, secondary]).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    })
}

pub fn list_workspaces(conn: &Connection) -> Result<Vec<WorkspaceItem>, String> {
    let mut statement = conn
        .prepare(
            "SELECT id, name, icon, accent_primary, accent_secondary
             FROM workspaces ORDER BY position ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(WorkspaceItem {
                id: row.get(0)?,
                name: row.get(1)?,
                icon: row.get(2)?,
                accent_primary: row.get(3)?,
                accent_secondary: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn create_workspace(conn: &Connection, workspace: &WorkspaceItem) -> Result<(), String> {
    validate_workspace(conn, workspace, None)?;
    let position: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(position) + 1, 0) FROM workspaces",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO workspaces (id, name, name_key, icon, accent_primary, accent_secondary, position)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            workspace.id,
            workspace.name.trim(),
            workspace.name.trim().to_lowercase(),
            workspace.icon,
            workspace.accent_primary,
            workspace.accent_secondary,
            position
        ],
    )
    .map_err(|e| map_constraint_error(e, "A workspace with this name or ID already exists"))?;
    Ok(())
}

pub fn update_workspace(conn: &Connection, workspace: &WorkspaceItem) -> Result<(), String> {
    validate_workspace(conn, workspace, Some(&workspace.id))?;
    let changed = conn
        .execute(
            "UPDATE workspaces
             SET name = ?2, name_key = ?3, icon = ?4, accent_primary = ?5, accent_secondary = ?6
             WHERE id = ?1",
            params![
                workspace.id,
                workspace.name.trim(),
                workspace.name.trim().to_lowercase(),
                workspace.icon,
                workspace.accent_primary,
                workspace.accent_secondary
            ],
        )
        .map_err(|e| map_constraint_error(e, "A workspace with this name already exists"))?;
    if changed == 0 {
        return Err("Workspace not found".to_string());
    }
    Ok(())
}

pub fn delete_workspace(conn: &Connection, workspace_id: &str) -> Result<(), String> {
    if workspace_id == "ws-personal" {
        return Err("The Personal workspace cannot be deleted".to_string());
    }
    let changed = conn
        .execute("DELETE FROM workspaces WHERE id = ?1", [workspace_id])
        .map_err(|e| e.to_string())?;
    if changed == 0 {
        return Err("Workspace not found".to_string());
    }
    compact_positions(conn)
}

fn compact_positions(conn: &Connection) -> Result<(), String> {
    let workspaces = list_workspaces(conn)?;
    for (position, workspace) in workspaces.iter().enumerate() {
        conn.execute(
            "UPDATE workspaces SET position = ?2 WHERE id = ?1",
            params![workspace.id, position as i64],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn validate_workspace(
    conn: &Connection,
    workspace: &WorkspaceItem,
    excluding_id: Option<&str>,
) -> Result<(), String> {
    let name = workspace.name.trim();
    if name.is_empty() {
        return Err("Workspace name cannot be empty".to_string());
    }
    if name.chars().count() > 40 {
        return Err("Workspace name must be 40 characters or fewer".to_string());
    }
    if workspace.id.trim().is_empty() {
        return Err("Workspace ID cannot be empty".to_string());
    }
    for color in [&workspace.accent_primary, &workspace.accent_secondary] {
        let bytes = color.as_bytes();
        let valid =
            bytes.len() == 7 && bytes[0] == b'#' && bytes[1..].iter().all(u8::is_ascii_hexdigit);
        if !valid {
            return Err("Workspace colors must use the #RRGGBB format".to_string());
        }
    }

    let duplicate: Option<String> = conn
        .query_row(
            "SELECT id FROM workspaces WHERE name_key = ?1",
            [name.to_lowercase()],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if duplicate
        .as_deref()
        .is_some_and(|id| Some(id) != excluding_id)
    {
        return Err("A workspace with this name already exists".to_string());
    }
    Ok(())
}

fn map_constraint_error(error: rusqlite::Error, message: &str) -> String {
    match error {
        rusqlite::Error::SqliteFailure(code, _)
            if code.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            message.to_string()
        }
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn database() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        initialize_workspace_table(&conn).unwrap();
        conn
    }

    fn custom_workspace(id: &str, name: &str) -> WorkspaceItem {
        WorkspaceItem {
            id: id.to_string(),
            name: name.to_string(),
            icon: "📁".to_string(),
            accent_primary: "#6e8efb".to_string(),
            accent_secondary: "#a777e3".to_string(),
        }
    }

    #[test]
    fn seeds_defaults_once_and_preserves_customizations() {
        let conn = database();
        let mut personal = list_workspaces(&conn).unwrap().remove(0);
        personal.name = "My Personal".to_string();
        update_workspace(&conn, &personal).unwrap();
        initialize_workspace_table(&conn).unwrap();
        let list = list_workspaces(&conn).unwrap();
        assert_eq!(list.len(), 4);
        assert_eq!(list[0].id, "ws-personal");
        assert_eq!(list[0].name, "My Personal");
    }

    #[test]
    fn seeds_default_workspaces_with_neutral_colors() {
        let workspaces = list_workspaces(&database()).unwrap();
        let colors: Vec<(&str, &str, &str)> = workspaces
            .iter()
            .map(|item| {
                (
                    item.id.as_str(),
                    item.accent_primary.as_str(),
                    item.accent_secondary.as_str(),
                )
            })
            .collect();
        assert_eq!(
            colors,
            [
                ("ws-personal", "#f5f5f5", "#a3a3a3"),
                ("ws-work", "#e5e5e5", "#8a8a8a"),
                ("ws-study", "#d4d4d4", "#999999"),
                ("ws-gaming", "#c7c7c7", "#737373"),
            ]
        );
    }

    #[test]
    fn migrates_only_legacy_default_colors_once() {
        let conn = database();
        conn.pragma_update(None, "user_version", 0).unwrap();
        conn.execute(
            "DELETE FROM app_migrations WHERE name = 'workspace-palette-v1'",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE workspaces SET accent_primary = '#6e8efb', accent_secondary = '#a777e3'
             WHERE id = 'ws-personal'",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE workspaces SET accent_primary = '#123456', accent_secondary = '#6366f1'
             WHERE id = 'ws-work'",
            [],
        )
        .unwrap();
        create_workspace(&conn, &custom_workspace("ws-custom-1", "Custom")).unwrap();

        initialize_workspace_table(&conn).unwrap();
        let workspaces = list_workspaces(&conn).unwrap();
        let personal = workspaces
            .iter()
            .find(|item| item.id == "ws-personal")
            .unwrap();
        assert_eq!(personal.accent_primary, "#f5f5f5");
        assert_eq!(personal.accent_secondary, "#a3a3a3");

        let work = workspaces.iter().find(|item| item.id == "ws-work").unwrap();
        assert_eq!(work.accent_primary, "#123456");
        assert_eq!(work.accent_secondary, "#8a8a8a");

        let custom = workspaces
            .iter()
            .find(|item| item.id == "ws-custom-1")
            .unwrap();
        assert_eq!(custom.accent_primary, "#6e8efb");
        assert_eq!(custom.accent_secondary, "#a777e3");

        conn.execute(
            "UPDATE workspaces SET accent_primary = '#6e8efb' WHERE id = 'ws-personal'",
            [],
        )
        .unwrap();
        initialize_workspace_table(&conn).unwrap();
        let personal = list_workspaces(&conn).unwrap().remove(0);
        assert_eq!(personal.accent_primary, "#6e8efb");
    }

    #[test]
    fn creates_updates_and_rejects_duplicate_names_case_insensitively() {
        let conn = database();
        let custom = custom_workspace("ws-custom-1", "Creative");
        create_workspace(&conn, &custom).unwrap();
        let mut renamed = custom.clone();
        renamed.name = "Design Lab".to_string();
        update_workspace(&conn, &renamed).unwrap();
        assert_eq!(
            list_workspaces(&conn).unwrap().last().unwrap().name,
            "Design Lab"
        );
        assert!(create_workspace(&conn, &custom_workspace("ws-custom-2", "dEsIgN lAb")).is_err());
    }

    #[test]
    fn protects_personal_and_compacts_order_after_deletion() {
        let conn = database();
        assert!(delete_workspace(&conn, "ws-personal").is_err());
        create_workspace(&conn, &custom_workspace("ws-custom-1", "Extra")).unwrap();
        delete_workspace(&conn, "ws-work").unwrap();
        let list = list_workspaces(&conn).unwrap();
        assert_eq!(
            list.iter().map(|ws| ws.id.as_str()).collect::<Vec<_>>(),
            ["ws-personal", "ws-study", "ws-gaming", "ws-custom-1"]
        );
        create_workspace(&conn, &custom_workspace("ws-custom-2", "Another")).unwrap();
        assert_eq!(
            list_workspaces(&conn).unwrap().last().unwrap().id,
            "ws-custom-2"
        );
    }
}
