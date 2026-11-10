use super::{
    browser_state::{BrowserState, ViewportBounds},
    tab_model::TabInfo,
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SplitLayout {
    pub workspace_id: String,
    pub left_tab_id: String,
    pub right_tab_id: String,
    pub ratio: f64,
    pub focused_tab_id: String,
    #[serde(default)]
    pub left_tab_ids: Vec<String>,
    #[serde(default)]
    pub right_tab_ids: Vec<String>,
}
impl SplitLayout {
    pub fn contains(&self, id: &str) -> bool {
        self.left_tab_id == id || self.right_tab_id == id
    }
    /// Existing tabs select their owning pane; new tabs join the focused group.
    pub fn focus_tab(&mut self, id: &str) {
        let side = self.side_of(id).unwrap_or_else(|| self.focused_side());
        if !self.group(side).iter().any(|tab| tab == id) {
            self.group_mut(side).push(id.to_string());
        }
        *self.selected_mut(side) = id.to_string();
        self.focused_tab_id = id.to_string();
    }
    pub fn key(&self) -> String {
        format!(
            "{}:{}:{}",
            self.workspace_id, self.left_tab_id, self.right_tab_id
        )
    }
    pub fn valid(&self, tabs: &[TabInfo]) -> bool {
        self.left_tab_id != self.right_tab_id
            && self.ratio.is_finite()
            && self.contains(&self.focused_tab_id)
            && [&self.left_tab_id, &self.right_tab_id].iter().all(|id| {
                tabs.iter()
                    .any(|t| &t.id == *id && t.workspace_id == self.workspace_id)
            })
            && ((self.left_tab_ids.is_empty() && self.right_tab_ids.is_empty())
                || self.valid_groups(tabs))
    }
}
pub fn pane_bounds(bounds: ViewportBounds, layout: &SplitLayout) -> [(String, ViewportBounds); 2] {
    let available = (bounds.width - 8.0).max(200.0);
    let left = (available * layout.ratio.clamp(0.2, 0.8)).clamp(100.0, available - 100.0);
    [
        (
            layout.left_tab_id.clone(),
            ViewportBounds {
                width: left,
                ..bounds
            },
        ),
        (
            layout.right_tab_id.clone(),
            ViewportBounds {
                x: bounds.x + left + 8.0,
                width: available - left,
                ..bounds
            },
        ),
    ]
}
impl BrowserState {
    /// Live geometry only. Session snapshots and persistence always use current_split().
    pub fn display_split(&self) -> Option<SplitLayout> {
        let mut layout = self.current_split()?;
        if let Some((key, ratio)) = self.session().edit_split_preview().lock().ok()?.as_ref() {
            if *key == layout.key() {
                layout.ratio = *ratio;
            }
        }
        Some(layout)
    }

    pub fn preview_split_ratio(&self, key: &str, ratio: f64) -> Result<(), String> {
        if !ratio.is_finite() || !(0.2..=0.8).contains(&ratio) {
            return Err("Invalid split ratio".into());
        }
        let layout = self
            .current_split()
            .filter(|l| l.key() == key)
            .ok_or("Split view changed")?;
        *self
            .session()
            .edit_split_preview()
            .lock()
            .map_err(|e| e.to_string())? =
            (ratio != layout.ratio).then(|| (key.to_string(), ratio));
        Ok(())
    }

    pub fn clear_split_preview(&self, key: &str) -> Result<(), String> {
        let mut preview = self
            .session()
            .edit_split_preview()
            .lock()
            .map_err(|e| e.to_string())?;
        if preview.as_ref().is_some_and(|(current, _)| current == key) {
            *preview = None;
        }
        Ok(())
    }

    pub fn current_split(&self) -> Option<SplitLayout> {
        let ws = self
            .session()
            .edit_active_workspace_id()
            .lock()
            .ok()?
            .clone();
        self.session()
            .edit_split_layouts()
            .lock()
            .ok()?
            .get(&ws)
            .cloned()
    }
    pub fn refresh_visible_tabs(&self) -> Result<(), String> {
        let workspace = self
            .session()
            .edit_active_workspace_id()
            .lock()
            .map_err(|e| e.to_string())?
            .clone();
        let mut tabs = self
            .session()
            .edit_tabs()
            .lock()
            .map_err(|e| e.to_string())?;
        let mut layouts = self
            .session()
            .edit_split_layouts()
            .lock()
            .map_err(|e| e.to_string())?;
        layouts.retain(|_, layout| {
            layout.normalize(&tabs);
            layout.valid(&tabs)
        });
        for tab in tabs.iter_mut() {
            tab.is_visible = tab.is_active
                || layouts
                    .get(&workspace)
                    .is_some_and(|layout| layout.contains(&tab.id));
            if tab.is_visible {
                tab.is_hibernated = false;
            }
        }
        Ok(())
    }
}
#[cfg(test)]
#[path = "split_groups_tests.rs"]
mod group_tests;
#[path = "split_groups.rs"]
mod groups;
#[cfg(test)]
#[path = "split_preview_tests.rs"]
mod preview_tests;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selection_replaces_only_focused_pane_and_moving_a_member_dissolves_pair() {
        let state = BrowserState::default();
        let workspace = "ws-personal";
        for id in ["b", "c"] {
            state
                .session()
                .edit_tabs()
                .lock()
                .unwrap()
                .push(TabInfo::new(
                    id.into(),
                    "https://example.test/".into(),
                    workspace.into(),
                    false,
                ));
        }
        state.session().edit_split_layouts().lock().unwrap().insert(
            workspace.into(),
            SplitLayout {
                left_tab_ids: Vec::new(),
                right_tab_ids: Vec::new(),
                workspace_id: workspace.into(),
                left_tab_id: "tab-1".into(),
                right_tab_id: "b".into(),
                focused_tab_id: "tab-1".into(),
                ratio: 0.6,
            },
        );
        state.refresh_visible_tabs().unwrap();
        assert_eq!(
            state
                .snapshot_tabs()
                .iter()
                .filter(|t| t.is_visible)
                .count(),
            2
        );
        state.select_tab("c").unwrap();
        state.refresh_visible_tabs().unwrap();
        assert_eq!(state.current_split().unwrap().left_tab_id, "c");
        assert_eq!(state.current_split().unwrap().right_tab_id, "b");
        assert!(state
            .snapshot_tabs()
            .iter()
            .filter(|t| t.is_visible)
            .all(|t| t.protected_from_hibernation()));
        state.move_tab("b", "ws-work").unwrap();
        state.refresh_visible_tabs().unwrap();
        assert!(state.current_split().is_none());
    }
    #[test]
    fn geometry_has_gap_and_preserves_minimum_sizes() {
        let layout = SplitLayout {
            left_tab_ids: Vec::new(),
            right_tab_ids: Vec::new(),
            workspace_id: "ws".into(),
            left_tab_id: "a".into(),
            right_tab_id: "b".into(),
            focused_tab_id: "a".into(),
            ratio: 0.2,
        };
        let rect = ViewportBounds {
            x: 10.0,
            y: 20.0,
            width: 380.0,
            height: 600.0,
        };
        let panes = pane_bounds(rect, &layout);
        assert_eq!(panes[0].1.width, 100.0);
        assert_eq!(panes[1].1.x, 118.0);
        assert_eq!(panes[1].1.width, 272.0);
    }
}
