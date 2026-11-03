use super::{
    browser_state::BrowserState, split_layout::SplitLayout, tab_checkpoint::TabCheckpoint,
    tab_model::TabInfo,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DockSide {
    Left,
    Right,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockContext {
    pub workspace_id: String,
    pub active_tab_id: String,
    pub split_key: Option<String>,
}

pub fn layout(
    tabs: &[TabInfo],
    current: Option<SplitLayout>,
    context: &DockContext,
    tab_id: &str,
    side: DockSide,
) -> Result<SplitLayout, String> {
    if !tabs
        .iter()
        .any(|tab| tab.id == tab_id && tab.workspace_id == context.workspace_id)
    {
        return Err("Dragged tab is no longer in this workspace".into());
    }
    let (left, right, ratio) = if let Some(split) = current {
        if !split.valid(tabs) {
            return Err("Split view changed".into());
        }
        match side {
            DockSide::Left if split.right_tab_id == tab_id => {
                (tab_id.into(), split.left_tab_id, split.ratio)
            }
            DockSide::Right if split.left_tab_id == tab_id => {
                (split.right_tab_id, tab_id.into(), split.ratio)
            }
            DockSide::Left => (tab_id.into(), split.right_tab_id, split.ratio),
            DockSide::Right => (split.left_tab_id, tab_id.into(), split.ratio),
        }
    } else {
        let companion = if context.active_tab_id != tab_id {
            tabs.iter().find(|tab| {
                tab.id == context.active_tab_id && tab.workspace_id == context.workspace_id
            })
        } else {
            // Strict greater-than retains the original tab order on timestamp ties.
            tabs.iter()
                .filter(|tab| tab.workspace_id == context.workspace_id && tab.id != tab_id)
                .fold(None::<&TabInfo>, |best, tab| {
                    if best.is_none_or(|old| tab.last_active_epoch > old.last_active_epoch) {
                        Some(tab)
                    } else {
                        best
                    }
                })
        }
        .ok_or("Choose another tab in this workspace to split")?;
        match side {
            DockSide::Left => (tab_id.into(), companion.id.clone(), 0.5),
            DockSide::Right => (companion.id.clone(), tab_id.into(), 0.5),
        }
    };
    Ok(SplitLayout {
        workspace_id: context.workspace_id.clone(),
        left_tab_id: left,
        right_tab_id: right,
        ratio,
        focused_tab_id: tab_id.into(),
    })
}

pub fn validate_context(
    state: &BrowserState,
    context: &DockContext,
    tab_id: &str,
) -> Result<(), String> {
    let workspace = state
        .active_workspace_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let active = state
        .active_tab_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let current = state.current_split();
    if workspace != context.workspace_id
        || active != context.active_tab_id
        || current.as_ref().map(SplitLayout::key) != context.split_key
    {
        return Err("Tab drag context changed".into());
    }
    if !state
        .snapshot_tabs()
        .iter()
        .any(|tab| tab.id == tab_id && tab.workspace_id == context.workspace_id)
    {
        return Err("Dragged tab is no longer in this workspace".into());
    }
    Ok(())
}

pub fn plan(
    state: &BrowserState,
    context: &DockContext,
    tab_id: &str,
    side: DockSide,
) -> Result<SplitLayout, String> {
    validate_context(state, context, tab_id)?;
    layout(
        &state.snapshot_tabs(),
        state.current_split(),
        context,
        tab_id,
        side,
    )
}

/// Caller holds tab_operations. No native document changes until this commits.
pub fn commit(
    state: &BrowserState,
    context: &DockContext,
    tab_id: &str,
    side: DockSide,
) -> Result<String, String> {
    let next = plan(state, context, tab_id, side)?;
    let activation = (context.active_tab_id != tab_id).then_some((tab_id, false));
    TabCheckpoint::commit_with_activation(state, activation, || {
        state
            .split_layouts
            .lock()
            .map_err(|e| e.to_string())?
            .insert(context.workspace_id.clone(), next);
        Ok(state.select_tab(tab_id)?.url)
    })
}

#[cfg(test)]
#[path = "split_docking_tests.rs"]
mod tests;
