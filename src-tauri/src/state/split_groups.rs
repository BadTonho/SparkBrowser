use super::SplitLayout;
use crate::state::{split_docking::DockSide, tab_model::TabInfo};
use std::collections::HashSet;

impl SplitLayout {
    pub fn focused_side(&self) -> DockSide {
        if self.focused_tab_id == self.right_tab_id {
            DockSide::Right
        } else {
            DockSide::Left
        }
    }
    pub fn group(&self, side: DockSide) -> &[String] {
        match side {
            DockSide::Left => &self.left_tab_ids,
            DockSide::Right => &self.right_tab_ids,
        }
    }
    pub fn group_mut(&mut self, side: DockSide) -> &mut Vec<String> {
        match side {
            DockSide::Left => &mut self.left_tab_ids,
            DockSide::Right => &mut self.right_tab_ids,
        }
    }
    pub fn selected_mut(&mut self, side: DockSide) -> &mut String {
        match side {
            DockSide::Left => &mut self.left_tab_id,
            DockSide::Right => &mut self.right_tab_id,
        }
    }
    pub fn side_of(&self, id: &str) -> Option<DockSide> {
        if self.left_tab_ids.iter().any(|tab| tab == id) || self.left_tab_id == id {
            Some(DockSide::Left)
        } else if self.right_tab_ids.iter().any(|tab| tab == id) || self.right_tab_id == id {
            Some(DockSide::Right)
        } else {
            None
        }
    }
    pub fn groups_key(&self) -> String {
        serde_json::to_string(&(&self.left_tab_ids, &self.right_tab_ids))
            .expect("group IDs serialize")
    }
    pub fn valid_groups(&self, tabs: &[TabInfo]) -> bool {
        let expected: HashSet<_> = tabs
            .iter()
            .filter(|t| t.workspace_id == self.workspace_id)
            .map(|t| &t.id)
            .collect();
        let all: Vec<_> = self
            .left_tab_ids
            .iter()
            .chain(&self.right_tab_ids)
            .collect();
        let unique: HashSet<_> = all.iter().copied().collect();
        !self.left_tab_ids.is_empty()
            && !self.right_tab_ids.is_empty()
            && self.left_tab_ids.contains(&self.left_tab_id)
            && self.right_tab_ids.contains(&self.right_tab_id)
            && unique.len() == all.len()
            && unique == expected
    }
    /// Legacy layouts put other tabs on the left. Later arrivals join the focused side.
    pub fn normalize(&mut self, tabs: &[TabInfo]) {
        let legacy = self.left_tab_ids.is_empty() && self.right_tab_ids.is_empty();
        if legacy {
            self.left_tab_ids.push(self.left_tab_id.clone());
            self.right_tab_ids.push(self.right_tab_id.clone());
        }
        let focused = self.focused_side();
        let left: HashSet<_> = self.left_tab_ids.iter().cloned().collect();
        let right: HashSet<_> = self.right_tab_ids.iter().cloned().collect();
        self.left_tab_ids.clear();
        self.right_tab_ids.clear();
        let workspace = self.workspace_id.clone();
        for tab in tabs.iter().filter(|t| t.workspace_id == workspace) {
            let side = if left.contains(&tab.id) {
                DockSide::Left
            } else if right.contains(&tab.id) {
                DockSide::Right
            } else if legacy {
                DockSide::Left
            } else {
                focused
            };
            self.group_mut(side).push(tab.id.clone());
        }
        for side in [DockSide::Left, DockSide::Right] {
            let selected = self.selected_mut(side).clone();
            if !self.group(side).contains(&selected) {
                if let Some(first) = self.group(side).first().cloned() {
                    *self.selected_mut(side) = first;
                }
            }
        }
        if !self.contains(&self.focused_tab_id) {
            self.focused_tab_id = self.selected_mut(focused).clone();
        }
    }
    /// Call before removing a tab from the workspace, so the nearest neighbor is known.
    pub fn remove_member(&mut self, id: &str) -> bool {
        let focused = self.focused_side();
        for side in [DockSide::Left, DockSide::Right] {
            if let Some(index) = self.group(side).iter().position(|tab| tab == id) {
                self.group_mut(side).remove(index);
                if self.selected_mut(side) == id {
                    if let Some(next) = self
                        .group(side)
                        .get(index)
                        .or_else(|| self.group(side).last())
                        .cloned()
                    {
                        *self.selected_mut(side) = next;
                    }
                }
            }
        }
        if self.focused_tab_id == id {
            self.focused_tab_id = self.selected_mut(focused).clone();
        }
        !self.left_tab_ids.is_empty() && !self.right_tab_ids.is_empty()
    }
    pub fn dock_member(&mut self, id: &str, side: DockSide) {
        let other = if side == DockSide::Left {
            DockSide::Right
        } else {
            DockSide::Left
        };
        // Only when the other pane would be left empty do the two shown tabs trade places (keeping the split).
        // Otherwise the tab just changes side and the pane it left shows its next tab.
        if self.selected_mut(other) == id && self.group(other).len() == 1 {
            let replaced = self.selected_mut(side).clone();
            for member in self.group_mut(other) {
                if member == id {
                    *member = replaced.clone();
                }
            }
            for member in self.group_mut(side) {
                if *member == replaced {
                    *member = id.to_string();
                }
            }
            *self.selected_mut(other) = replaced;
        } else if self.side_of(id) != Some(side) {
            self.remove_member(id);
            self.group_mut(side).push(id.to_string());
        }
        *self.selected_mut(side) = id.to_string();
        self.focused_tab_id = id.to_string();
    }
}
