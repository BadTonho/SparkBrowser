import assert from 'node:assert/strict';
import { createNativeKeySender } from './native-keyboard.mjs';
import { withScenarioDatabase } from './persistence/database.mjs';

export async function checkTabDividers(context) {
  const { invoke, script, click, until, fixture, restart, withWebview } = context;
  if (!await script('return !!document.querySelector(".tabs-list")')) {
    await context.openSettings();
    await click('[data-testid=settings-appearance_layout]');
    await click('#setting-browser-layout .mini-pill:nth-child(2)');
    await context.closeSettings();
  }

  const create = async name => {
    const { tabs } = await invoke('create_tab', { url: fixture.origin + '/page/' + name });
    return tabs.find(tab => tab.is_active).id;
  };
  const tabSelector = id => `.tab-row[data-tab-id="${id}"]`;
  const dividers = () => invoke('get_tab_dividers');
  const addDivider = async (id, name) => {
    const row = tabSelector(id);
    await until(() => script('return !!document.querySelector(arguments[0])', row), 'divider anchor is rendered');
    await script(`
      const target = document.querySelector(arguments[0]);
      const bounds = target.getBoundingClientRect();
      target.dispatchEvent(new MouseEvent('contextmenu', {
        bubbles: true, cancelable: true, button: 2, clientX: bounds.x + 10, clientY: bounds.y + 5,
      }));
    `, row);
    await withWebview('/bookmark-context-menu.html', () => until(() => script(`
      const button = [...document.querySelectorAll('[role=menuitem]')].find(
        item => ['Add divider below', 'Adicionar divisão abaixo'].includes(item.textContent.trim()),
      );
      if (!button?.getClientRects().length) return false;
      button.click();
      return true;
    `), 'add divider menu action'));
    await until(() => script('return !!document.querySelector(".tab-divider input")'), 'divider name editor');
    await script(`
      const input = document.querySelector('.tab-divider input');
      input.value = arguments[0];
      input.dispatchEvent(new Event('input', { bubbles: true }));
      input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }));
    `, name);
    return until(async () => (await dividers()).find(item => item.after === id && item.name === name), 'named divider persisted');
  };
  const verifyDivider = async (divider, after) => {
    assert.equal((await dividers()).find(item => item.id === divider.id)?.after, after);
    await until(() => script(`
      const divider = document.querySelector('.tab-divider[data-tab-id="divider:' + arguments[0] + '"]');
      const previous = document.querySelector('.tab-row[data-tab-id="' + arguments[1] + '"]');
      if (!divider?.checkVisibility()) return false;
      const order = Number(getComputedStyle(divider).order);
      return arguments[1] ? previous && order === Number(getComputedStyle(previous).order) + 1 : order === 1;
    `, divider.id, after), 'divider remains visible at its new position');
  };

  const above = await create('divider-above');
  const anchor = await create('divider-shortcut');
  const shortcutDivider = await addDivider(anchor, 'Pesquisa');
  await withScenarioDatabase(context.directory, async db => {
    db.exec(`CREATE TRIGGER fail_divider_write BEFORE UPDATE ON app_preferences
      WHEN NEW.key = 'tab_dividers_v1' BEGIN SELECT RAISE(ABORT, 'injected divider write failure'); END`);
    try {
      await assert.rejects(invoke('close_tab', { tabId: anchor }), /injected divider write failure/);
      assert.ok((await invoke('get_tabs')).tabs.some(tab => tab.id === anchor));
      await verifyDivider(shortcutDivider, anchor);
    } finally {
      db.exec('DROP TRIGGER fail_divider_write');
    }
  });
  if (process.platform === 'win32') {
    await context.focusWindow();
    await click('.omnibox-input');
    const key = createNativeKeySender(context.directory);
    await key(0x11);
    try { await key.press(0x57); } finally { await key(0x11, true); }
    await until(async () => !(await invoke('get_tabs')).tabs.some(tab => tab.id === anchor), 'Ctrl+W closes anchor');
  } else {
    await invoke('close_tab', { tabId: anchor });
  }
  await verifyDivider(shortcutDivider, above);

  const sidebarAnchor = await create('divider-sidebar');
  const sidebarDivider = await addDivider(sidebarAnchor, 'Projetos');
  await click(tabSelector(sidebarAnchor) + ' .close-btn');
  await until(async () => !(await invoke('get_tabs')).tabs.some(tab => tab.id === sidebarAnchor), 'sidebar button closes anchor');
  await verifyDivider(sidebarDivider, above);

  const first = (await invoke('get_tabs')).tabs[0].id;
  const bulkAnchor = await create('divider-bulk');
  const bulkDivider = await addDivider(bulkAnchor, 'Leitura');
  await create('divider-selected');
  const { closed } = await invoke('close_tabs_reviewed', { tabIds: [first, above, bulkAnchor] });
  assert.deepEqual(new Set(closed), new Set([first, above, bulkAnchor]));
  await verifyDivider(bulkDivider, '');

  await restart();
  await verifyDivider(shortcutDivider, '');
  await verifyDivider(sidebarDivider, '');
  await verifyDivider(bulkDivider, '');
}
