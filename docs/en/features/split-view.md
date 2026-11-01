# Split view

In **Tools → Tabs**, select two different tabs from the current workspace and choose **Split selected tabs**. Pages appear side by side with an 8 px divider. Drag the divider or focus it and use Left/Right; Home restores 50%. Ratios range from 20% to 80%, with each native view at least 100 logical pixels wide.

Clicking a page focuses its tab and navigation controls. Selecting another tab replaces the focused pane. Exiting split view keeps the focused tab. Closing or moving a paired tab dissolves the pair. Each workspace remembers its pair, ratio and focused tab across restarts; inactive workspaces may hibernate their pages. The current pair is protected against manual/automatic hibernation and bulk closing, even when the resident limit is 1.

Overlays hide both views and restore them afterward. Geometry updates reuse resident WebViews without navigating or changing focus; pointer movement is coalesced and the ratio persists when adjustment finishes. Windows uses native focus events; other engines use the generation-checked page focus bridge and require platform validation.
