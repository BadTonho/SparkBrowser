import { createSharedLifecycle } from "../../services/lifecycle.js";
import { onNavigationUpdated, onTabsUpdated } from "../../services/bridge/events";
import { applyNavigationPatch, applyTabsSnapshot, initializeTabs } from "./actions";
import { initializeWorkspaces } from "../workspaceStore";

let ready: Promise<void> = Promise.resolve();
const retain = createSharedLifecycle(scope => {
  // Register before fetching: live events win against slower initial snapshots.
  ready = Promise.all([
    onNavigationUpdated(scope.guard(applyNavigationPatch)).then(scope.own),
    onTabsUpdated(scope.guard(applyTabsSnapshot)).then(scope.own),
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
