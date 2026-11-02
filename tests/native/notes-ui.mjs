import assert from 'node:assert/strict';
export async function checkNotesUi(invoke, until, script, origin) {
  await script(`const button = document.querySelector('[data-testid="open-tools"]'); button.focus(); button.click();`);
  await until(() => script(`return Boolean(document.querySelector('[data-testid="tools-dialog"]'))`), 'Tools dialog');
  await script(`document.querySelector('[data-testid="tools-notes"]').click();`);
  await until(() => script(`return Boolean(document.querySelector('[data-testid="page-note-editor"]'))`), 'page note editor');
  await script(`const editor = document.querySelector('textarea'); editor.value = 'Artificial native editor draft'; editor.dispatchEvent(new Event('input', { bubbles: true }));`);
  const url = origin + '/page/a/spa';
  await until(async () => (await invoke('get_page_note', { url }))?.text === 'Artificial native editor draft', 'debounced note saved');
  assert.equal(await script(`return document.querySelector('textarea').value`), 'Artificial native editor draft');
  await script(`document.querySelector('[data-testid="close-tools"]').click();`);
  await until(() => script(`return !document.querySelector('[data-testid="tools-dialog"]')`), 'Tools closed');
  await invoke('delete_page_note', { url });
}
