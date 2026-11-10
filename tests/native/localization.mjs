import assert from 'node:assert/strict';

export async function checkLocalization(context) {
  const { script, until, restart, invoke, click, withWebview } = context;
  const original = (await invoke('get_tabs')).tabs;
  const openSettings = async () => {
    await context.openSettings();
    await until(() => script(`return Boolean(document.querySelector('[data-testid="settings-appearance_layout"]'))`), 'settings navigation');
    await click('[data-testid="settings-appearance_layout"]');
  };
  const language = value => withWebview('/index.html?surface=settings', () => script(`const select = document.querySelector('[data-testid="interface-language"]'); select.value = arguments[0]; select.dispatchEvent(new Event('change', { bubbles: true }));`, value));
  await openSettings();
  await until(() => script(`return Boolean(document.querySelector('[data-testid="interface-language"]'))`), 'language preference');
  await language('pt-BR');
  await until(() => script(`return document.documentElement.lang === 'pt-BR' && document.querySelector('[data-testid="open-tools"]').getAttribute('aria-label') === 'Ferramentas'`), 'Portuguese immediately');
  assert.equal(await script(`return document.querySelector('[data-testid="interface-language"]').options[0].textContent`), 'Sistema');
  await context.closeSettings();
  await script(`document.querySelector('[data-testid="open-tools"]').click();`);
  await until(() => script(`return Boolean(document.querySelector('[data-testid="tools-notes"]'))`), 'tools navigation');
  await script(`document.querySelector('[data-testid="tools-notes"]').click();`);
  await until(() => script(`return document.querySelector('[data-testid="page-note-editor"]')?.getAttribute('aria-label') === 'Nota da página atual'`), 'Portuguese tools');
  await script(`document.querySelector('[data-testid="close-tools"]').click();`);
  await restart();
  await until(() => script(`return document.documentElement.lang === 'pt-BR'`), 'language persisted');
  await openSettings(); await language('en');
  await until(() => script(`return document.querySelector('[data-testid="open-tools"]').getAttribute('aria-label') === 'Tools'`), 'English immediately');
  const current = (await invoke('get_tabs')).tabs;
  assert.deepEqual(current.map(tab => [tab.id, tab.workspace_id, tab.url]), original.map(tab => [tab.id, tab.workspace_id, tab.url]));
}
