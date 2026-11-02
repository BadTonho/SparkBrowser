import assert from 'node:assert/strict';

/** Real toolbar interaction in both layouts and in the focused split pane. */
export async function checkPipToolbarLayouts({invoke,until,click,script,fixture},id) {
  const floating=value=>until(async()=>{
    const tab=(await invoke('get_tab_media')).tabs.find(tab=>tab.tab_id===id);
    return Boolean(tab?.frames.some(frame=>frame.picture_in_picture))===value;
  },`toolbar floating state ${value}`);
  const enabled=()=>until(()=>script('return document.querySelector("[data-testid=toggle-pip]")?.disabled===false;'),'PiP toolbar ready');
  for (const layout of [1,0]) {
    await click('[data-testid=open-settings]');await click('[data-testid=settings-appearance_layout]');
    await script('document.querySelectorAll("#setting-browser-layout button")[arguments[0]].click();',layout);
    await script('document.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}));');
    await enabled();
    await click('[data-testid=toggle-pip]');await floating(true);
    await enabled();await click('[data-testid=toggle-pip]');await floating(false);
  }

  const other=(await invoke('create_tab',{url:fixture.origin+'/page/pip-split-peer'})).tabs.find(tab=>tab.is_active).id;
  await until(()=>fixture.reports.has('pip-split-peer'),'split peer ready');
  await invoke('set_split_view',{leftTabId:id,rightTabId:other});
  await invoke('switch_tab',{tabId:other});
  await until(()=>script('return document.querySelector("[data-testid=toggle-pip]")?.disabled===true;'),'empty focused pane disables PiP');
  await invoke('switch_tab',{tabId:id});await enabled();
  await click('[data-testid=toggle-pip]');await floating(true);
  await invoke('switch_tab',{tabId:other});await enabled();
  assert.equal(await script('return document.querySelector("[data-testid=toggle-pip]").getAttribute("aria-pressed");'),'true','global PiP stays active in the other pane');
  await click('[data-testid=toggle-pip]');await floating(false);
  await invoke('set_split_view',{leftTabId:null,rightTabId:null});
  await invoke('switch_tab',{tabId:id});
  await invoke('close_tab',{tabId:other});
}
