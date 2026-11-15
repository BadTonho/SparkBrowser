import assert from 'node:assert/strict';

export async function checkPanelIdleRelease(context) {
  const { click, request, script, sessionId, until, withWebview } = context;
  const path = `/session/${sessionId()}/window`;
  const inTools = action => withWebview('/index.html?surface=tools', action);
  const open = async () => {
    await context.focusWindow();
    await click('[data-action=tools]');
    await until(() => script('return !!document.querySelector(".tools-controller")'), 'tools host opens');
    return inTools(async () => {
      await until(() => script('return !!document.querySelector("[data-testid=tools-dialog]")'), 'tools surface opens');
      return request('GET', path);
    });
  };

  const original = await open();
  await inTools(() => script('document.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}))'));
  await until(() => script('return !document.querySelector(".tools-controller")'), 'tools host closes');
  await until(async () => !(await request('GET', `${path}/handles`)).includes(original), 'idle tools WebView is released', 45_000);
  assert.notEqual(await open(), original, 'the tools panel reopens in a fresh WebView');
}
