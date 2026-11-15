/** @typedef {'popup' | 'tab'} DisplayMode */

/** @template {{url:string,workspace_id:string}} T @param {T[]} tabs @param {string} workspace @returns {T | undefined} */
export function findSettingsTab(tabs, workspace) {
  return tabs.find(tab => tab.url === 'spark://settings' && tab.workspace_id === workspace);
}

/** Coordinates the durable preference and the visible settings surface.
 * @param {{ getMode: () => DisplayMode, tabs: () => {id:string, is_active:boolean, url:string, workspace_id:string}[], workspace: () => string, save: (mode:DisplayMode)=>boolean, set: (mode:DisplayMode)=>void, openTab:()=>Promise<string>, closeTab:(id:string)=>Promise<void>, selectTab:(id:string)=>Promise<void>, popupOpen:()=>boolean, closePopup:()=>Promise<void>, openPopup:()=>Promise<void> }} options
 */
export function createSettingsDisplayController(options) {
  let busy = false;
  return {
    /** @param {DisplayMode} mode */
    async change(mode) {
      if (mode !== 'popup' && mode !== 'tab') throw new Error('Invalid settings display mode');
      if (busy) throw new Error('Settings display change already in progress');
      const previous = options.getMode();
      const currentSettings = findSettingsTab(options.tabs(), options.workspace());
      const needsTransfer = mode === 'tab' ? options.popupOpen() : !options.popupOpen() && !!currentSettings?.is_active;
      if (previous === mode && !needsTransfer) return;
      busy = true;
      const initial = options.tabs();
      const initialIds = new Set(initial.map(tab => tab.id));
      const originalActive = initial.find(tab => tab.is_active)?.id;
      const popupWasOpen = options.popupOpen();
      let openedTab, removedTab = false;
      try {
        if (!options.save(mode)) throw new Error('Could not save settings display preference');
        try {
          if (mode === 'tab') {
            openedTab = await options.openTab();
            await options.closePopup();
          } else {
            const tab = findSettingsTab(options.tabs(), options.workspace());
            if (tab) { await options.closeTab(tab.id); removedTab = true; }
            await options.openPopup();
          }
          options.set(mode);
        } catch (error) {
          options.save(previous);
          try {
            if (mode === 'tab') {
              if (openedTab && !initialIds.has(openedTab)) await options.closeTab(openedTab);
              if (originalActive && options.tabs().some(tab => tab.id === originalActive)) await options.selectTab(originalActive);
              if (popupWasOpen && !options.popupOpen()) await options.openPopup();
            } else if (removedTab) {
              if (options.popupOpen()) {
                try { await options.closePopup(); } catch { /* Still try to restore the tab. */ }
              }
              await options.openTab();
            }
          } catch { /* Preserve the original error for the caller. */ }
          throw error;
        }
      } finally { busy = false; }
    },
  };
}
