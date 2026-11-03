use super::{
    model::*,
    validation::{page_url, tab_url},
};
use std::collections::{BTreeMap, HashMap, HashSet};

pub struct MergePlan {
    pub data: BackupData,
    pub added: BTreeMap<String, usize>,
    pub skipped: Vec<BackupConflict>,
    pub next_tab_counter: u64,
}
pub fn counts(d: &BackupData) -> BTreeMap<String, usize> {
    [
        ("workspaces", d.workspaces.len()),
        ("folders", d.bookmarks.folders.len()),
        ("bookmarks", d.bookmarks.bookmarks.len()),
        ("sessions", d.sessions.len()),
        ("notes", d.notes.len()),
        ("site_search", d.site_search.len()),
        ("tabs", d.tabs.len()),
        ("split_layouts", d.split_layouts.len()),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v))
    .collect()
}
fn skip(rows: &mut Vec<BackupConflict>, kind: &str, key: &str, reason: &str) {
    rows.push(BackupConflict {
        kind: kind.into(),
        key: key.into(),
        reason: reason.into(),
    });
}
fn page_key(
    session: &crate::database::saved_sessions_repo::SavedSession,
) -> Vec<(String, String, bool, bool)> {
    session
        .pages
        .iter()
        .map(|page| {
            (
                page.title.clone(),
                page.url.clone(),
                page.is_pinned,
                page.is_selected,
            )
        })
        .collect()
}
pub fn plan(
    current: &BackupData,
    incoming: &BackupData,
    token: &str,
    mut counter: u64,
) -> Result<MergePlan, String> {
    let mut merged = current.clone();
    let before = counts(current);
    let mut skipped = Vec::new();
    let mut workspace_names: HashMap<_, _> = current
        .workspaces
        .iter()
        .map(|v| (v.name.trim().to_lowercase(), v.id.clone()))
        .collect();
    let mut workspace_ids: HashSet<_> = current.workspaces.iter().map(|v| v.id.clone()).collect();
    let mut workspace_map = HashMap::new();
    let mut new_workspaces = HashSet::new();
    for (index, ws) in incoming.workspaces.iter().enumerate() {
        let name = ws.name.trim().to_lowercase();
        let id = if let Some(existing) = workspace_names.get(&name) {
            skip(&mut skipped, "workspaces", &ws.name, "existing");
            existing.clone()
        } else {
            let mut item = ws.clone();
            item.id = format!("import-{token}-{index}");
            while workspace_ids.contains(&item.id) {
                item.id.push('x');
            }
            let id = item.id.clone();
            new_workspaces.insert(id.clone());
            workspace_ids.insert(id.clone());
            workspace_names.insert(name, id.clone());
            merged.workspaces.push(item);
            id
        };
        workspace_map.insert(ws.id.clone(), id);
    }
    let mut folder_names: HashMap<_, _> = current
        .bookmarks
        .folders
        .iter()
        .map(|v| (v.name.trim().to_lowercase(), v.id))
        .collect();
    let mut folder_map = HashMap::new();
    let mut folder_id = merged
        .bookmarks
        .folders
        .iter()
        .map(|v| v.id)
        .max()
        .unwrap_or(0)
        + 1;
    for folder in &incoming.bookmarks.folders {
        let name = folder.name.trim().to_lowercase();
        let id = if let Some(existing) = folder_names.get(&name) {
            skip(&mut skipped, "folders", &folder.name, "existing");
            *existing
        } else {
            let mut item = folder.clone();
            item.id = folder_id;
            folder_names.insert(name, folder_id);
            folder_id += 1;
            merged.bookmarks.folders.push(item);
            folder_id - 1
        };
        folder_map.insert(folder.id, id);
    }
    let mut bookmark_urls: HashSet<_> = merged
        .bookmarks
        .bookmarks
        .iter()
        .map(|v| page_url(&v.url).unwrap_or_else(|_| v.url.clone()))
        .collect();
    let mut bookmark_id = merged
        .bookmarks
        .bookmarks
        .iter()
        .map(|v| v.id)
        .max()
        .unwrap_or(0)
        + 1;
    for item in &incoming.bookmarks.bookmarks {
        let url = page_url(&item.url)?;
        if !bookmark_urls.insert(url.clone()) {
            skip(&mut skipped, "bookmarks", &url, "existing");
            continue;
        }
        let mut item = item.clone();
        item.id = bookmark_id;
        bookmark_id += 1;
        item.url = url;
        item.folder_id = item.folder_id.and_then(|id| folder_map.get(&id).copied());
        merged.bookmarks.bookmarks.push(item);
    }
    let mut note_urls: HashSet<_> = merged.notes.iter().map(|n| n.url.clone()).collect();
    for note in &incoming.notes {
        let url = crate::database::notes_repo::normalize(&note.url)?;
        if !note_urls.insert(url.clone()) {
            skip(&mut skipped, "notes", &url, "existing");
            continue;
        }
        let mut note = note.clone();
        note.url = url;
        merged.notes.push(note);
    }
    let mut keywords: HashSet<_> = merged
        .site_search
        .iter()
        .map(|n| n.keyword.to_lowercase())
        .collect();
    for search in &incoming.site_search {
        if !keywords.insert(search.keyword.to_lowercase()) {
            skip(&mut skipped, "site_search", &search.keyword, "existing");
            continue;
        }
        merged.site_search.push(search.clone());
    }
    let mut session_id = merged.sessions.iter().map(|v| v.id).max().unwrap_or(0) + 1;
    let mut page_sets: HashSet<_> = current.sessions.iter().map(page_key).collect();
    let mut session_names: HashSet<_> = current
        .sessions
        .iter()
        .map(|s| s.name.to_lowercase())
        .collect();
    let mut suffixes: HashMap<String, usize> = HashMap::new();
    for session in &incoming.sessions {
        if !page_sets.insert(page_key(session)) {
            skip(&mut skipped, "sessions", &session.name, "identical");
            continue;
        }
        let mut item = session.clone();
        item.id = session_id;
        session_id += 1;
        let base = item.name.trim().to_string();
        let mut suffix = *suffixes.get(&base.to_lowercase()).unwrap_or(&2);
        while session_names.contains(&item.name.to_lowercase()) {
            let tail = format!(" ({suffix})");
            item.name = base
                .chars()
                .take(100 - tail.chars().count())
                .collect::<String>()
                + &tail;
            suffix += 1;
        }
        suffixes.insert(base.to_lowercase(), suffix);
        session_names.insert(item.name.to_lowercase());
        merged.sessions.push(item);
    }
    // Map occurrences, not a URL set: deliberate duplicate tabs remain duplicates.
    let mut occurrences: HashMap<(String, String), Vec<String>> = HashMap::new();
    for tab in &current.tabs {
        occurrences
            .entry((
                tab.workspace_id.clone(),
                tab_url(&tab.url).unwrap_or_else(|_| tab.url.clone()),
            ))
            .or_default()
            .push(tab.id.clone());
    }
    let mut seen: HashMap<(String, String), usize> = HashMap::new();
    let mut tab_map = HashMap::new();
    let mut ids: HashSet<_> = current
        .tabs
        .iter()
        .chain(&incoming.tabs)
        .map(|t| t.id.clone())
        .collect();
    for tab in &incoming.tabs {
        let workspace = workspace_map
            .get(&tab.workspace_id)
            .ok_or("backup.invalid_workspace")?
            .clone();
        let url = tab_url(&tab.url)?;
        let key = (workspace.clone(), url.clone());
        let occurrence = seen.entry(key.clone()).or_default();
        if let Some(id) = occurrences.get(&key).and_then(|v| v.get(*occurrence)) {
            tab_map.insert(tab.id.clone(), id.clone());
            skip(&mut skipped, "tabs", &url, "existing");
        } else {
            let id = loop {
                let id = format!("tab-{counter}");
                counter = counter.checked_add(1).ok_or("backup.id_limit")?;
                if ids.insert(id.clone()) {
                    break id;
                }
            };
            let mut item = tab.clone();
            item.id = id.clone();
            item.workspace_id = workspace;
            item.url = url;
            item.is_active = false;
            tab_map.insert(tab.id.clone(), id);
            merged.tabs.push(item);
        }
        *occurrence += 1;
    }
    for layout in &incoming.split_layouts {
        let workspace = workspace_map
            .get(&layout.workspace_id)
            .ok_or("backup.invalid_workspace")?;
        if !new_workspaces.contains(workspace)
            || merged
                .split_layouts
                .iter()
                .any(|l| &l.workspace_id == workspace)
        {
            skip(&mut skipped, "split_layouts", workspace, "existing");
            continue;
        }
        let mut item = layout.clone();
        item.normalize(
            &incoming
                .tabs
                .iter()
                .map(super::model::BackupTab::tab)
                .collect::<Vec<_>>(),
        );
        for id in item.left_tab_ids.iter_mut().chain(&mut item.right_tab_ids) {
            *id = tab_map.get(id).ok_or("backup.invalid_split")?.clone();
        }
        item.workspace_id = workspace.clone();
        item.left_tab_id = tab_map
            .get(&layout.left_tab_id)
            .ok_or("backup.invalid_split")?
            .clone();
        item.right_tab_id = tab_map
            .get(&layout.right_tab_id)
            .ok_or("backup.invalid_split")?
            .clone();
        item.focused_tab_id = tab_map
            .get(&layout.focused_tab_id)
            .ok_or("backup.invalid_split")?
            .clone();
        merged.split_layouts.push(item);
    }
    let added = counts(&merged)
        .into_iter()
        .map(|(k, v)| {
            let delta = v - before[&k];
            (k, delta)
        })
        .collect();
    Ok(MergePlan {
        data: merged,
        added,
        skipped,
        next_tab_counter: counter,
    })
}
