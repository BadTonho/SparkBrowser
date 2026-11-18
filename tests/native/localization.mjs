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
  // The tools panel is a native window above the page; its content is read and clicked inside it.
  const tools = action => withWebview('/index.html?surface=tools', action);
  await script(`document.querySelector('[data-testid="open-tools"]').click();`);
  await until(() => script(`return Boolean(document.querySelector('.tools-controller'))`), 'tools host opens');
  await until(() => tools(() => script(`return Boolean(document.querySelector('[data-testid="tools-notes"]'))`)), 'tools navigation');
  // The panel's own labels are in the chosen language (the note editor needs an ordinary web page, which this scenario does not open).
  await until(() => tools(() => script(`return document.querySelector('[data-testid="close-tools"]')?.getAttribute('aria-label') === 'Fechar ferramentas' && document.querySelector('[data-testid="tools-dialog"]')?.getAttribute('aria-label') === 'Ferramentas'`)), 'Portuguese tools');
  await tools(() => click('[data-testid="close-tools"]'));
  await until(() => script(`return !document.querySelector('.tools-controller')`), 'tools host closes');
  await restart();
  await until(() => script(`return document.documentElement.lang === 'pt-BR'`), 'language persisted');
  await openSettings(); await language('en');
  await until(() => script(`return document.querySelector('[data-testid="open-tools"]').getAttribute('aria-label') === 'Tools'`), 'English immediately');
  const current = (await invoke('get_tabs')).tabs;
  assert.deepEqual(current.map(tab => [tab.id, tab.workspace_id, tab.url]), original.map(tab => [tab.id, tab.workspace_id, tab.url]));
}
