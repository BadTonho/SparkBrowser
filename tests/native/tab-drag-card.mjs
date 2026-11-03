import assert from 'node:assert/strict';

// Inspect the native card before any real gesture. Switching WebDriver's
// selected window while a pointer is held would itself cancel capture.
export async function checkDragCard(context, tab) {
  const { invoke, script, until, withWebview, screenshot } = context;
  const snapshot = await invoke('get_tabs');
  const point = await script('return {x:Math.round(innerWidth/2),y:Math.round(innerHeight/2)}');
  const presentation = { tabId: tab.id, side: null, context: {
    workspaceId: tab.workspace_id, activeTabId: tab.id, splitKey: null,
  }, label: '', locale: 'en', pointer: point };
  // Low revisions are reserved here before frontend gestures start. Normal
  // shell updates start at Date.now(), so these calls cannot suppress them.
  await invoke('set_tab_drag_preview', { revision: 1, presentation });
  try {
    await withWebview('/tab-drag-preview.html', async () => {
      const card = await until(() => script(`
        const ghost=document.querySelector('#dragged-tab'), r=ghost.getBoundingClientRect();
        const title=document.querySelector('#dragged-tab-title').textContent;
        return title?{title,host:document.querySelector('#dragged-tab-host').textContent,
          x:r.x,y:r.y,width:r.width,height:r.height,paneHidden:document.querySelector('#split-drop-preview').hidden}:null;
      `), 'native dragged tab card');
      assert.equal(card.title, tab.title);
      assert.equal(card.host, '127.0.0.1');
      assert.equal(card.paneHidden, true, 'single tab still gets a drag card without offering split');
      assert.ok(Math.abs(card.x-point.x-16)<=1 && Math.abs(card.y-point.y-18)<=1);
      assert.ok(card.width>=200 && card.height>=48);
      await assert.rejects(invoke('get_tabs'), /restricted|not allowed|denied/i);
      await screenshot('dragged-tab-card');
    });
  } finally { await invoke('set_tab_drag_preview', { revision: 2, presentation: null }); }
  assert.deepEqual((await invoke('get_tabs')).split_layout, snapshot.split_layout);
  assert.deepEqual((await invoke('get_tabs')).tabs.map(tab=>tab.id), snapshot.tabs.map(tab=>tab.id));
}
