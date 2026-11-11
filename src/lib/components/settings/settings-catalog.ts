import { type SettingsCategory } from "../../stores/settingsStore";
import { SHORTCUT_DEFINITIONS } from "../../stores/shortcutsStore";

export const categories: {
  id: SettingsCategory;
  icon: string;
  label: string;
}[] = [
    { id: "data_backup", icon: "⇩", label: "Data & Backup" },
    { id: "updates", icon: "↻", label: "About & Updates" },
    { id: "diagnostics", icon: "◷", label: "Local Diagnostics" },
    { id: "appearance_layout", icon: "monitor", label: "Appearance & Layout" },
    { id: "search_navigation", icon: "search", label: "Search & Navigation" },
    { id: "workspaces", icon: "palette", label: "Workspaces & Theme" },
    { id: "performance_privacy", icon: "rocket", label: "Performance" },
    { id: "privacy_security", icon: "shield", label: "Privacy & Security" },
    { id: "shortcuts", icon: "keyboard", label: "Keyboard Shortcuts" },
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
  {id:"pip-window-shadow",title:"Floating window shadow",description:"Show a shadow around the floating video window.",category:"appearance_layout",targetId:"setting-pip-window-shadow",keywords:"pip picture in picture floating video window shadow sombra janela video flutuante"},
  {id:"whatsapp-display-mode",title:"WhatsApp Display",description:"Choose a compact popover or a side panel that resizes the page.",category:"appearance_layout",targetId:"setting-whatsapp-display-mode",keywords:"whatsapp popover compact side panel layout janela painel lateral compacto"},
  { id: "theme-mode", title: "Theme Mode", description: "Choose between dark and light appearance.", category: "appearance_layout", targetId: "setting-theme-mode", keywords: "theme dark light modo tema escuro claro appearance aparencia" },
  { id: "wide-address-bar", title: "Wide Address Bar", description: "Use all available toolbar space for the address bar.", category: "appearance_layout", targetId: "setting-wide-address-bar", keywords: "wide large address bar omnibox largura barra endereço" },
  { id: "full-urls", title: "Always Show Full URLs", description: "Keep the protocol and www visible when the address bar is idle.", category: "appearance_layout", targetId: "setting-full-urls", keywords: "url urls protocol https www complete completo endereço" },
  { id: "compact-toolbar", title: "Compact Toolbar", description: "Use the smaller toolbar dimensions.", category: "appearance_layout", targetId: "setting-compact-toolbar", keywords: "compact density small toolbar modo compacto barra" },
  { id: "rounded-content-frame", title: "Rounded Content Corners", description: "Show or hide the frame around the main WebView area.", category: "appearance_layout", targetId: "setting-rounded-content-frame", keywords: "rounded corners border frame webview cantos arredondados borda moldura" },
  { id: "data-backup", title: "Data & Backup", description: "Export or merge a local backup.", category: "data_backup", targetId: "backup-heading", keywords: "backup import export restore data dados importar exportar restaurar sync sincronizacao google drive onedrive mega pendrive webdav" },
  { id: "updates", title: "About & Updates", description: "Check for signed releases and choose Stable or Beta.", category: "updates", targetId: "updates-heading", keywords: "version upgrade update github release channel stable beta" },
  { id: "site-permissions", title: "Site Permissions", description: "Control saved camera, microphone, location and notification decisions.", category: "privacy_security", targetId: "site-permissions-heading", keywords: "origin camera microphone location notifications allow block ask" },
  { id: "clear-local-history", title: "Clear local browsing history", description: "Delete the browser's local navigation history without changing site data.", category: "privacy_security", targetId: "clear-local-history", keywords: "history delete clear visited sites recently closed" },
  { id: "history-retention", title: "Local history retention", description: "Choose how long local browsing history is kept.", category: "privacy_security", targetId: "setting-history-retention", keywords: "history retention expire month months forever" },
  { id: "popup-setting", title: "Pop-ups", description: "Allow web pages to open links in new browser tabs.", category: "privacy_security", targetId: "setting-popup-policy", keywords: "popup pop-ups windows new tab block allow" },
  { id: "smart-screen", title: "Microsoft Defender SmartScreen", description: "Use WebView2 reputation checking for phishing and malware protection.", category: "privacy_security", targetId: "setting-smart-screen", keywords: "safe browsing dangerous sites phishing malware reputation windows" },
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
    keywords: "fullscreen focus ctrl shift z",
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
    category: "privacy_security",
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
  { id: "hibernate-exceptions", title: "Hibernation exceptions", description: "Keep selected domains and subdomains out of automatic hibernation.", category: "performance_privacy", targetId: "setting-hibernation-exceptions", keywords: "performance memory ram domain exclude exception" },
  { id: "ask-download-destination", title: "Ask download location", description: "Choose a destination before each download starts.", category: "performance_privacy", targetId: "setting-download-destination", keywords: "download save folder path location" },
  { id: "site-zoom", title: "Page zoom", description: "Reset page zoom", category: "search_navigation", targetId: "zoom-heading", keywords: "zoom origin site scale" },
  ...SHORTCUT_DEFINITIONS.map((shortcut) => ({
    id: `setting-${shortcut.id}`,
    title: shortcut.label,
    description: shortcut.description,
    category: "shortcuts" as const,
    targetId: `shortcut-${shortcut.id}`,
    keywords: "keyboard shortcut",
  })),
];
