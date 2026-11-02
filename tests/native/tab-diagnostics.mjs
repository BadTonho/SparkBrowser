import assert from 'node:assert/strict';
import { prepareTabs } from './setup.mjs';
export async function checkTabDiagnostics(context) {
  const { invoke, until, fixture, click, script } = context;
  const { a, b } = await prepareTabs(context);
  await invoke('set_split_view',{leftTabId:a.id,rightTabId:b.id});
  let diagnostics = await invoke('get_diagnostics');
  assert.equal(diagnostics.tab_details.filter(t=>t.resident).length,diagnostics.resident_webviews);
  assert.ok(diagnostics.tab_details.find(t=>t.id===a.id).protection_reasons.includes('form'));
  for (const id of [a.id,b.id]) assert.ok(diagnostics.tab_details.find(t=>t.id===id).protection_reasons.includes('split'));
  await click('[data-testid="open-tools"]');
  await until(()=>script('return Boolean(document.querySelector("[data-testid=tools-diagnostics]"))'),'diagnostics tab');
  await click('[data-testid="tools-diagnostics"]');
  await until(()=>script('return Boolean(document.querySelector("[data-testid=diagnostics-totals]"))'),'diagnostics panel');
  assert.ok(await script('return document.querySelectorAll("[data-testid=diagnostic-tab]").length>=2'));
  await click('[data-testid="close-tools"]');
  await invoke('set_split_view',{leftTabId:null,rightTabId:null});
  await invoke('set_tab_pinned',{tabId:b.id,pinned:false});
  const extra=(await invoke('create_tab',{url:fixture.origin+'/page/diagnostic-extra'})).tabs.find(t=>t.is_active);
  await until(()=>fixture.reports.has('diagnostic-extra'),'extra page');
  await invoke('switch_tab',{tabId:a.id});
  diagnostics=await invoke('get_diagnostics');
  assert.deepEqual(diagnostics.tab_details.find(t=>t.id===extra.id).protection_reasons,[]);
  await click('[data-testid="open-tools"]');
  await until(()=>script('return Boolean(document.querySelector("[data-testid=tools-diagnostics]"))'),'tools reopen');
  await click('[data-testid="tools-diagnostics"]');
  await until(()=>script('return Boolean(document.querySelector(arguments[0]))', `[data-tab-id="${extra.id}"] [data-testid="diagnostic-hibernate"]`),'extra diagnostic');
  // Protection can change after a row was fetched. The backend must reject it.
  await invoke('set_tab_pinned',{tabId:extra.id,pinned:true});
  await assert.rejects(()=>invoke('hibernate_tab',{tabId:extra.id}), /protected/);
  await invoke('set_tab_pinned',{tabId:extra.id,pinned:false});
  await click(`[data-tab-id="${extra.id}"] [data-testid="diagnostic-hibernate"]`);
  await until(async()=>!(await invoke('get_diagnostics')).tab_details.find(t=>t.id===extra.id).resident,'actual webview released');
  await until(()=>script('return document.querySelector(arguments[0]).disabled', `[data-tab-id="${extra.id}"] [data-testid="diagnostic-hibernate"]`),'polled hibernated row');
}
