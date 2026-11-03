use super::{
    browser_state::BrowserState,
    split_docking::{self, DockContext, DockSide},
    tab_checkpoint::TabCheckpoint,
};

pub fn commit(
    state: &BrowserState,
    context: &DockContext,
    id: &str,
    side: DockSide,
    before_id: Option<&str>,
) -> Result<(), String> {
    split_docking::validate_context(state, context, id)?;
    if context.groups_key.is_none() {
        return Err("Missing split group context".into());
    }
    let mut next = state.current_split().ok_or("Split view changed")?;
    let source = next.side_of(id).ok_or("Tab group changed")?;
    if before_id == Some(id) && source == side {
        return Ok(());
    }
    if before_id.is_some_and(|before| !next.group(side).iter().any(|tab| tab == before)) {
        return Err("Destination tab changed".into());
    }
    let cross = source != side;
    let selections = (
        next.left_tab_id.clone(),
        next.right_tab_id.clone(),
        next.focused_tab_id.clone(),
    );
    let keep_split = next.remove_member(id);
    let group = next.group_mut(side);
    let index = before_id
        .and_then(|before| group.iter().position(|tab| tab == before))
        .unwrap_or(group.len());
    group.insert(index, id.into());
    if cross {
        *next.selected_mut(side) = id.into();
        next.focused_tab_id = id.into();
    } else {
        (next.left_tab_id, next.right_tab_id, next.focused_tab_id) = selections;
    }
    let order = next.group(side).to_vec();
    let activation = (cross && context.active_tab_id != id).then_some((id, false));
    TabCheckpoint::commit_with_activation(state, activation, || {
        {
            let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
            let by_id: std::collections::HashMap<_, _> = tabs
                .iter()
                .map(|tab| (tab.id.clone(), tab.clone()))
                .collect();
            let mut reordered = order.iter();
            for tab in tabs.iter_mut().filter(|tab| order.contains(&tab.id)) {
                *tab = by_id[reordered.next().expect("same group length")].clone();
            }
            next.normalize(&tabs);
        }
        let mut layouts = state.split_layouts.lock().map_err(|e| e.to_string())?;
        if keep_split || !cross {
            layouts.insert(context.workspace_id.clone(), next);
        } else {
            layouts.remove(&context.workspace_id);
        }
        drop(layouts);
        if cross {
            state.select_tab(id)?;
        }
        Ok(())
    })
}
