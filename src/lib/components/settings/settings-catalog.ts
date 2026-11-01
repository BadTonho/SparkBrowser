import { type SettingsCategory } from "../../stores/settingsStore";
import { SHORTCUT_DEFINITIONS } from "../../stores/shortcutsStore";

export const categories: {
  id: SettingsCategory;
  icon: string;
  label: string;
}[] = [
    { id: "diagnostics", icon: "◷", label: "Local Diagnostics" },
    { id: "appearance_layout", icon: "🖥️", label: "Appearance & Layout" },
    { id: "search_navigation", icon: "🔎", label: "Search & Navigation" },
    { id: "workspaces", icon: "🎨", label: "Workspaces & Theme" },
    { id: "performance_privacy", icon: "🚀", label: "Performance & Privacy" },
    { id: "shortcuts", icon: "⌨️", label: "Keyboard Shortcuts" },
  ];

export type SearchEntry = {
  id: string;
  title: string;
  description: string;
  category: SettingsCategory;
  targetId: string;
  keywords?: string;
};

export const searchableSettings: SearchEntry[] = [
  { id: "site-permissions", title: "Site Permissions", description: "Control saved camera, microphone, location and notification decisions.", category: "performance_privacy", targetId: "site-permissions-heading", keywords: "origin camera microphone location notifications allow block ask" },
  { id: "site-search", title: "Site Search Shortcuts", description: "Configure keyword searches for websites.", category: "search_navigation", targetId: "site-search-heading", keywords: "keyword template custom search" },
  { id: "resident-limit", title: "Resident Tab Limit", description: "Choose how many eligible WebViews stay awake.", category: "performance_privacy", targetId: "setting-resident-limit", keywords: "memory ram lru pinned forms media downloads" },
  { id: "local-diagnostics", title: "Local Diagnostics", description: "Inspect IPC timings, failures and session rows written.", category: "diagnostics", targetId: "diagnostics-heading", keywords: "performance latency p95 metrics database" },
  {
    id: "browser-layout",
    title: "Browser Layout",
    description: "Choose a horizontal browser layout or a vertical sidebar layout.",
    category: "appearance_layout",
    targetId: "setting-browser-layout",
    keywords: "horizontal vertical tabs sidebar workspaces bookmarks",
  },
  {
    id: "background-blur",
    title: "Background Blur",
    description: "Control the blur intensity behind browser overlays.",
    category: "appearance_layout",
    targetId: "setting-background-blur",
    keywords: "overlay intensity transparency glass 0 24 px",
  },
  {
    id: "bookmarks-bar",
    title: "Bookmarks Bar",
    description: "Show or hide the horizontal bookmarks bar below the address bar.",
    category: "appearance_layout",
    targetId: "setting-bookmarks-bar",
    keywords: "favorites show hide visibility toolbar horizontal",
  },
  {
    id: "workspace-switcher-display",
    title: "Workspace Switcher Display",
    description: "Choose horizontal workspace buttons or a floating Alt picker; vertical sidebar buttons stay visible.",
    category: "appearance_layout",
    targetId: "setting-workspace-switcher-display",
    keywords: "workspaces toolbar buttons popup floating window alt hold presentation",
  },
  {
    id: "zen-mode",
    title: "Zen / Focus Mode",
    description: "Hide browser controls so websites use the full viewport.",
    category: "appearance_layout",
    targetId: "setting-zen-mode",
    keywords: "fullscreen focus alt z",
  },
  {
    id: "default-search-engine",
    title: "Default Search Engine",
    description: "Used when typing search queries in the Omnibox.",
    category: "search_navigation",
    targetId: "setting-default-search-engine",
    keywords: "search omnibox web google duckduckgo engine",
  },
  {
    id: "workspace-profile",
    title: "Workspace Identity & Theme",
    description: "Choose a workspace to edit its name, emoji, colors, or manage profiles.",
    category: "workspaces",
    targetId: "setting-workspace-profiles",
    keywords: "workspaces personal work study gaming name rename emoji icon theme primary secondary accent colors create delete",
  },
  {
    id: "adblock-shield",
    title: "Privacy Shield",
    description: "Block ad domains and trackers at the network and DOM layers.",
    category: "performance_privacy",
    targetId: "setting-adblock-shield",
    keywords: "privacy ads trackers block protection",
  },
  {
    id: "auto-hibernate",
    title: "Auto-Hibernate Inactive Tabs",
    description: "Choose 5, 10, or 30 minutes, adaptive Auto mode, or turn automatic hibernation Off.",
    category: "performance_privacy",
    targetId: "setting-auto-hibernate",
    keywords: "performance memory ram 5m 10m 30m auto frequent frequently used 7 days 5 hours disabled off",
  },
  ...SHORTCUT_DEFINITIONS.map((shortcut) => ({
    id: `setting-${shortcut.id}`,
    title: shortcut.label,
    description: shortcut.description,
    category: "shortcuts" as const,
    targetId: `shortcut-${shortcut.id}`,
    keywords: "keyboard shortcut",
  })),
];
