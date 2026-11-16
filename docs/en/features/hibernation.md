# Automatic hibernation exceptions

## Hibernate other workspaces

**Settings → Performance → Hibernate other workspaces** is **on by default**. A few seconds after you switch workspace (three, so flipping through workspaces with `Alt + 1` … `Alt + 9` does not put each one to sleep on the way), the tabs of the workspaces you are not in are hibernated: their pages are closed to free memory and reload when you open them. Turning the option on runs it once for the current workspace.

It is independent of **Hibernate inactive tabs automatically**, so it also applies when that is **Off**; turn this option off to keep every workspace's pages loaded. Tabs that are playing media or in Picture-in-Picture, have an edited form, are downloading or loading, or are pinned stay awake, as do the exemption domains below. Only the window's own tabs are affected. The preference is local to the device and not part of backups.

To share or separate cookies and storage between workspaces, use **Settings → Workspaces & Theme → Isolate workspace content** (**Shared** by default, **Isolated** when on); see [workspace isolation](workspace-isolation.md).

## Hibernation exceptions

**Settings → Performance & Privacy → Hibernation exceptions** keeps listed domains and their subdomains out of automatic hibernation, including the resident-tab limit. Entries are hostnames without a scheme, path or port. The browser trims whitespace and a final dot, normalizes case and internationalized names, and matches on DNS label boundaries, so `example.com` covers `shop.example.com` but not `notexample.com`.

**Off means off.** With **Hibernate inactive tabs automatically** set to **Off**, no tab is put to sleep on its own, including by the resident-tab limit. The limit keeps its value (the field is greyed out) and applies again as soon as another mode is chosen. Manual hibernation always remains available.

The list does not change explicit hibernation commands. A user can still hibernate an exempt tab manually; regular protections such as the active-tab and edited-form checks continue to apply. Removing a domain makes its tabs eligible for a later automatic hibernation pass.

The list is stored with local preferences and included in backups. Older backups without the list restore an empty list.
