import { checkTabDocking } from './tab-docking.mjs';
import { checkTitlebarActions } from './titlebar-actions.mjs';
import { checkToolsPopover } from './tools-popover.mjs';
import { checkDownloadsPopover } from './downloads-popover.mjs';
import { checkDownloadsList } from './downloads-list.mjs';
import { checkSecurityBoundaries } from './security-boundaries.mjs';
import { checkWebviewRuntime } from './webview-runtime.mjs';
import { checkNewTab } from './new-tab.mjs';
import {checkNewTabWidgets} from './new-tab-widgets.mjs';
import {newTabDriver} from './new-tab-driver.mjs';
import { checkSplitTabGroups } from './split-tab-groups.mjs';
import { checkPrivacy } from './privacy.mjs';
import { checkMediaMute, checkMediaVolume, checkMediaPip } from "./media.mjs";
import { checkPipContinuity } from './pip-continuity.mjs';
import { checkPipPerformance } from './pip-performance.mjs';
import {checkPipShadow} from './pip-shadow.mjs';
import { checkKeyboard } from "./keyboard.mjs";
import { checkCommandSearch } from "./command-search.mjs";
import { checkZoom } from "./zoom.mjs";
import { checkTabDiagnostics } from "./tab-diagnostics.mjs";
import { checkTabChurn } from './tab-churn.mjs';
import { checkSplitPersistence } from './persistence/split.mjs';
import { checkWorkspaceDeletePersistence } from './persistence/workspaces.mjs';
import { checkShieldPersistence } from './persistence/shield.mjs';
import { checkShutdown } from "./shutdown.mjs";
import { checkBackup } from "./backup.mjs";
import { checkDownloads } from './downloads.mjs';
import { checkClosedTabs } from './closed-tabs.mjs';
import { checkEphemeralCleanup } from './ephemeral-cleanup.mjs';
import { checkAccountContainers } from './account-containers.mjs';
import { checkRecovery, checkRecoveryFailure } from './recovery.mjs';
import { checkLocalization } from "./localization.mjs";
import assert from 'node:assert/strict';
import { prepareTabs } from './setup.mjs';
import { checkNotesUi } from './notes-ui.mjs';
import { checkPermissions } from './permissions.mjs';
import { checkFeatures } from './features.mjs';
import { checkWorkspaceSwitcher } from './workspace-switcher.mjs';
import { checkQuickSettings } from './quick-settings.mjs';
import { checkFullPageCaptureProbe } from './capture-probe.mjs';
import { checkDarkModeProbe } from './dark-mode-probe.mjs';
import { checkCaretBrowsing } from './caret-browsing.mjs';
import { checkMinimumFontProbe } from './minimum-font-probe.mjs';
import { checkNetworkPrivacyBaseline } from './network-privacy.mjs';

export const scenarios = {
  'network-privacy-baseline': checkNetworkPrivacyBaseline,
  'capture-probe': checkFullPageCaptureProbe,
  'dark-mode-probe': checkDarkModeProbe,
  'caret-browsing': checkCaretBrowsing,
  'minimum-font-probe': checkMinimumFontProbe,
  'security-boundaries': checkSecurityBoundaries,
  'webview-runtime': checkWebviewRuntime,
  'tools-popover': checkToolsPopover,
  'downloads-popover': checkDownloadsPopover,
  'quick-settings': checkQuickSettings,
  'downloads-list': checkDownloadsList,
  'titlebar-actions': checkTitlebarActions,
  'new-tab': checkNewTab,
  'new-tab-widgets': context=>checkNewTabWidgets(context,newTabDriver(context)),
  'split-tab-groups': checkSplitTabGroups,
  'tab-docking': checkTabDocking,
  'privacy': checkPrivacy,
  'persistence-split': checkSplitPersistence,
  'persistence-workspaces': checkWorkspaceDeletePersistence,
  'persistence-shield': checkShieldPersistence,
  'tab-churn': checkTabChurn,
  "media-mute": checkMediaMute,
  "media-volume": checkMediaVolume,
  "media-pip": checkMediaPip,
  "media-pip-continuity": checkPipContinuity,
  "media-pip-fps": checkPipPerformance,
  "media-pip-shadow":checkPipShadow,
  keyboard: checkKeyboard,
  "command-search": checkCommandSearch,
  zoom: checkZoom,
  "tab-diagnostics": checkTabDiagnostics,
  shutdown: checkShutdown,
  downloads: checkDownloads,
  recovery: checkRecovery,
  'recovery-failure': checkRecoveryFailure,
  'closed-tabs': checkClosedTabs,
  'ephemeral-cleanup': checkEphemeralCleanup,
  'account-containers': checkAccountContainers,
  backup: async context => { await prepareTabs(context); await checkBackup(context); },
  localization: async context => { await prepareTabs(context); await checkLocalization(context); },
  navigation: async context => {
    const { invoke, until, fixture } = context;
    const { a } = await prepareTabs(context);
    if (process.platform === 'win32') {
      await invoke('toggle_adblock_shield');
      assert.equal(fixture.loads.get('a'), 1, 'live shield changes must preserve the document');
      await invoke('go_back');
      await until(async () => (await invoke('get_tabs')).tabs.find(tab => tab.id === a.id)?.url.endsWith('/page/a'), 'native back after shield update');
      await invoke('go_forward');
      await until(async () => (await invoke('get_tabs')).tabs.find(tab => tab.id === a.id)?.url.endsWith('/spa'), 'native forward after shield update');
      await invoke('toggle_adblock_shield');
    }

  },
  dialogs: async context => {
    await context.focusWindow();
    const { invoke, until, fixture, script, click, wait, screenshot, request, sessionId } = context;
    const { a, b } = await prepareTabs(context);
    // Draft and focus lifecycle are checked in the shell's DOM through its real event handlers.
    await until(() => script('return document.querySelector(".omnibox-input").value === arguments[0]', fixture.origin + '/page/a/spa'), 'frontend navigation settled');
    await click('.omnibox-input');
    await script(`const input = document.querySelector('.omnibox-input'); input.focus(); input.value = 'draft survives'; input.dispatchEvent(new Event('input', { bubbles: true }));`);
    assert.equal(await script('return document.activeElement === document.querySelector(".omnibox-input")'), true, 'address has focus');
    await invoke('reorder_tabs', { tabIds: [b.id, a.id] });
    await wait(150);
    assert.equal(await script('return document.querySelector(".omnibox-input").value'), 'draft survives');
    await script(`document.querySelector('.omnibox-input').dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));`);
    // Exercise real UI handlers without introducing test-only IPC or expanded permissions.
    for (const label of ['open-settings', 'open-downloads', 'open-history']) {
      await script(`const button = document.querySelector('button[data-testid="' + arguments[0] + '"]'); button.focus(); button.click();`, label);
      if (label === 'open-history') {
        assert.equal(await script('return document.querySelector(".browser-preview-fallback") === null'), true, 'history stays on top of the real page without replacing the viewport');
      }
      const inspect = operation => label === 'open-downloads'
        ? context.withWebview('/index.html?surface=downloads', operation)
        : label === 'open-history'
          ? context.withWebview('/history-overlay.html', operation)
          : operation();
      await inspect(async () => {
      const dialog = label === 'open-history' ? '[role=dialog]:not([hidden])' : '[role=dialog]';
      await until(() => script(`return Boolean(document.querySelector('${dialog}'))`), `${label} dialog`);
      await until(() => script(`return document.querySelector('${dialog}').contains(document.activeElement)`), 'initial dialog focus');
      assert.equal(await script(`const dialog = document.querySelector('[role=dialog]');
        const controls = [...dialog.querySelectorAll('input:not([disabled]),select:not([disabled]),textarea:not([disabled]),button:not([disabled]),summary,a[href],[tabindex="0"]')].filter(node => node.getClientRects().length && node.checkVisibility());
        controls.at(-1).focus(); document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true, cancelable: true }));
        return document.activeElement === controls[0];`), true, 'Tab wraps within the current dialog');
      await script('document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }))');
      await until(() => script(`return !document.querySelector('${dialog}')`), 'Escape closes dialog');
      });
      await until(() => script('return document.activeElement?.getAttribute("data-testid") === arguments[0]', label), 'return focus to opener');
    }
    await script(`document.querySelector('button[data-testid="open-settings"]').click();`);
    await script(`document.querySelector('[data-testid="settings-updates"]').click();`);
    await until(() => script('return Boolean(document.querySelector("#updates-heading"))'), 'updates settings');
    assert.equal(await script('return document.querySelector("#update-channel").disabled'), true, 'isolated native builds cannot update the production installation');
    assert.equal((await invoke('get_update_state')).supported, false);
    if (process.env.VELOCE_NATIVE_SCREENSHOTS === '1') {
      await screenshot('updates');
    }
    await script('document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }))');
    for (const [width, height] of [[900, 700], [1400, 900]]) {
      await request('POST', `/session/${sessionId()}/window/rect`, { width, height });
      await until(() => script('const box = document.querySelector(".viewport-frame").getBoundingClientRect(); return box.width >= 100 && box.height >= 100;'), 'viewport after resize');
    }

  },
  'workspace-switcher': async context => {
    const { a, b } = await prepareTabs(context);
    await checkWorkspaceSwitcher(context.invoke, context.until, context.fixture, a, b);
  },
  notes: async context => {
    await prepareTabs(context);
    await checkNotesUi(context.invoke, context.until, context.script, context.fixture.origin);
  },
  features: async context => {
    const { a, b } = await prepareTabs(context);
    await checkFeatures(context.invoke, context.until, context.fixture, a, b);
    await context.restart();
    const packet = await context.invoke('get_tabs');
    assert.equal(packet.split_layout.ratio, 0.65);
    assert.equal(packet.split_layout.left_tab_id, a.id);
    assert.equal(packet.tabs.find(tab => tab.id === b.id)?.is_pinned, true);
    assert.ok(packet.tabs.filter(tab => !tab.is_active).every(tab => tab.is_hibernated || tab.is_visible));
  },
  permissions: async context => {
    const { a, b } = await prepareTabs(context);
    await checkPermissions(context.invoke, context.until, context.fixture, a, b, context.script, context.withWebview, context.click);
    await context.restart();
    if (process.platform === 'win32') {
      assert.ok((await context.invoke('get_site_permissions')).settings.some(row => row.origin === context.fixture.origin && row.kind === 'microphone' && row.state === 'allow'));
    }
  },
  restart: async context => {
    const { a, b } = await prepareTabs(context);
    await context.invoke('set_update_channel', { channel: 'beta' });
    await context.restart();
    assert.equal((await context.invoke('get_update_state')).channel, 'beta');
    const packet = await context.invoke('get_tabs');
    assert.ok(packet.tabs.some(tab => tab.id === a.id));
    assert.equal(packet.tabs.find(tab => tab.id === b.id)?.is_pinned, true);
  },
};
