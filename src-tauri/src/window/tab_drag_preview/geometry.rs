use crate::state::{
    browser_state::ViewportBounds,
    split_docking::DockSide,
    split_layout::{pane_bounds, SplitLayout},
};

pub struct DockGeometry {
    pub destination: ViewportBounds,
    pub companion: (String, ViewportBounds),
}

impl DockGeometry {
    pub fn new(bounds: ViewportBounds, layout: &SplitLayout, side: DockSide) -> Self {
        let [left, right] = pane_bounds(bounds, layout);
        match side {
            DockSide::Left => Self {
                destination: left.1,
                companion: right,
            },
            DockSide::Right => Self {
                destination: right.1,
                companion: left,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{
        split_docking::{layout, DockContext},
        tab_model::TabInfo,
    };

    #[test]
    fn preview_matches_committed_geometry_for_creation_replacement_and_swap() {
        let tabs: Vec<_> = ["a", "b", "c"]
            .into_iter()
            .map(|id| {
                TabInfo::new(
                    id.into(),
                    "https://example.test/".into(),
                    "ws".into(),
                    id == "a",
                )
            })
            .collect();
        let split = SplitLayout {
            workspace_id: "ws".into(),
            left_tab_id: "a".into(),
            right_tab_id: "b".into(),
            focused_tab_id: "a".into(),
            ratio: 0.65,
        };
        for current in [None, Some(split)] {
            let context = DockContext {
                workspace_id: "ws".into(),
                active_tab_id: "a".into(),
                split_key: current.as_ref().map(SplitLayout::key),
            };
            for tab in ["a", "b", "c"] {
                for side in [DockSide::Left, DockSide::Right] {
                    let next = layout(&tabs, current.clone(), &context, tab, side).unwrap();
                    for bounds in [
                        ViewportBounds::default(),
                        ViewportBounds {
                            x: 0.0,
                            y: 80.0,
                            width: 720.0,
                            height: 600.0,
                        },
                    ] {
                        let preview = DockGeometry::new(bounds, &next, side);
                        let panes = pane_bounds(bounds, &next);
                        let index = if side == DockSide::Left { 0 } else { 1 };
                        assert_eq!(preview.destination, panes[index].1);
                        assert_eq!(preview.companion, panes[1 - index]);
                        assert_ne!(preview.companion.0, tab);
                        assert_eq!(next.ratio, if current.is_some() { 0.65 } else { 0.5 });
                    }
                }
            }
        }
    }
}
