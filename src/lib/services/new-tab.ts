import { createSharedLifecycle } from "./lifecycle.js";
import { derived } from 'svelte/store';
import { locale } from '../i18n';
import { defaultSearchEngine } from '../stores/searchEngineStore';
import { invoke, isTauriRuntime } from './bridge/runtime';

export const NEW_TAB_URL = 'veloce://newtab';
export const isNewTab = (url: string) => url === NEW_TAB_URL;

function startNewTab(): () => void {
  if (!isTauriRuntime()) return () => {};
  let queue = Promise.resolve();
  let disposed = false;
  const stop = derived([defaultSearchEngine, locale], ([engine, language]) => ({ engine, language })).subscribe(preferences => {
    queue = queue.then(async () => { if (!disposed) await invoke('set_new_tab_preferences', preferences); }).catch(error => console.error('New tab preferences failed', error));
  });
  return () => { disposed = true; stop(); };
}

export const initializeNewTab = createSharedLifecycle(scope => scope.own(startNewTab()));
