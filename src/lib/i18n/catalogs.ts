import mediaEn from "./catalogs/en/media.json";
import newtabEn from "./catalogs/en/newtab.json";
import searchEn from "./catalogs/en/search.json";
import zoomEn from "./catalogs/en/zoom.json";
import diagnosticsEn from "./catalogs/en/diagnostics.json";
import shutdownEn from "./catalogs/en/shutdown.json";
import recoveryEn from "./catalogs/en/recovery.json";
import closedEn from "./catalogs/en/closed.json";
import backupEn from "./catalogs/en/backup.json";
import commonEn from "./catalogs/en/common.json";
import bookmarksEn from "./catalogs/en/bookmarks.json";
import commandbarEn from "./catalogs/en/commandbar.json";
import downloadsEn from "./catalogs/en/downloads.json";
import historyEn from "./catalogs/en/history.json";
import permissionsEn from "./catalogs/en/permissions.json";
import privacyEn from "./catalogs/en/privacy.json";
import settingsEn from "./catalogs/en/settings.json";
import sidebarEn from "./catalogs/en/sidebar.json";
import statusEn from "./catalogs/en/status.json";
import tabsEn from "./catalogs/en/tabs.json";
import titlebarEn from "./catalogs/en/titlebar.json";
import toolsEn from "./catalogs/en/tools.json";
import viewportEn from "./catalogs/en/viewport.json";
import workspacesEn from "./catalogs/en/workspaces.json";
import extensionsEn from "./catalogs/en/extensions.json";
import readerEn from "./catalogs/en/reader.json";
import passwordsEn from "./catalogs/en/passwords.json";
import autofillEn from "./catalogs/en/autofill.json";
import profilesEn from "./catalogs/en/profiles.json";
import lockEn from "./catalogs/en/lock.json";

export const english = { ...newtabEn, ...mediaEn, ...searchEn, ...zoomEn, ...diagnosticsEn, ...shutdownEn, ...recoveryEn, ...closedEn, ...backupEn, ...commonEn, ...bookmarksEn, ...commandbarEn, ...downloadsEn, ...historyEn, ...permissionsEn, ...privacyEn, ...settingsEn, ...sidebarEn, ...statusEn, ...tabsEn, ...titlebarEn, ...toolsEn, ...viewportEn, ...workspacesEn, ...extensionsEn, ...readerEn, ...passwordsEn, ...autofillEn, ...profilesEn, ...lockEn };
export type MessageKey = keyof typeof english;

/** Portuguese is read only when it is the interface language: every interface window loads this
 * module, and most of them would never use the other half of the text. English stays here as the
 * fallback that is always ready. */
export const loadPortuguese = (): Promise<Record<MessageKey, string>> =>
  import("./catalogs-pt").then(module => module.portuguese);
