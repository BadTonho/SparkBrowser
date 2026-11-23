import { createSharedLifecycle } from "../../services/lifecycle.js";
import { onNavigationUpdated, onTabsUpdated } from "../../services/bridge/events";
import { applyNavigationPatch, applyTabsSnapshot, initializeTabs } from "./actions";
import { initializeWorkspaces } from "../workspaceStore";
import { refreshTabDividers } from "../tabDividersStore";
import { onTabDividersUpdated } from "../../services/bridge/tab-dividers";
import { isTauriRuntime } from "../../services/bridge/runtime";
import { logIgnored } from "../../services/ignored-error";

let ready: Promise<void> = Promise.resolve();
const retain = createSharedLifecycle(scope => {
  // Register before fetching: live events win against slower initial snapshots.
  ready = Promise.all([
    onNavigationUpdated(scope.guard(applyNavigationPatch)).then(scope.own),
    onTabsUpdated(scope.guard(applyTabsSnapshot)).then(scope.own),
    (async () => {
      if (isTauriRuntime()) {
        await onTabDividersUpdated(scope.guard(() => { void refreshTabDividers(); })).then(scope.own).catch(logIgnored);
      }
      if (scope.active()) await refreshTabDividers(true);
    })(),
  ]).then(async () => {
    if (!scope.active()) return;
    await initializeWorkspaces(scope.active);
    if (scope.active()) await initializeTabs(scope.active);
  });
});

export function initializeBrowserSession(): { stop: () => void; ready: Promise<void> } {
  const stop = retain();
  return { stop, ready: ready.catch(error => { stop(); throw error; }) };
}
