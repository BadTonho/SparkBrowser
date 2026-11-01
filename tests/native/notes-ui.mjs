import assert from 'node:assert/strict';
export async function checkNotesUi(invoke, until, script, origin) {
  await script(`const button = document.querySelector('button[aria-label="Tools"]'); button.focus(); button.click();`);
  await until(() => script(`return Boolean(document.querySelector('[role="dialog"][aria-label="Tools"]'))`), 'Tools dialog');
  await script(`const buttons = [...document.querySelectorAll('.tools-panel nav button')]; buttons.find(button => button.textContent === 'Notes').click();`);
  await until(() => script(`return Boolean(document.querySelector('textarea[aria-label="Note for current page"]'))`), 'page note editor');
  await script(`const editor = document.querySelector('textarea'); editor.value = 'Artificial native editor draft'; editor.dispatchEvent(new Event('input', { bubbles: true }));`);
  const url = origin + '/page/a/spa';
  await until(async () => (await invoke('get_page_note', { url }))?.text === 'Artificial native editor draft', 'debounced note saved');
  assert.equal(await script(`return document.querySelector('textarea').value`), 'Artificial native editor draft');
  await script(`document.querySelector('button[aria-label="Close tools"]').click();`);
  await until(() => script(`return !document.querySelector('[role="dialog"][aria-label="Tools"]')`), 'Tools closed');
  await invoke('delete_page_note', { url });
}
