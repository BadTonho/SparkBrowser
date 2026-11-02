<script lang="ts">
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import { describeError } from "../../services/user-errors.js";
  import { t } from "../../i18n";
  import { tabs, applyTabsSnapshot } from "../../stores/browserStore";
  import { activeWorkspaceId } from "../../stores/workspaceSelectionStore";
  import { splitLayout } from "../../stores/splitViewStore";
  import { setSplitView } from "../../services/bridge/split-view";
  let left = $state(""); let right = $state(""); let busy = $state(false); let error = $state("");
  const candidates = $derived($tabs.filter(tab => tab.workspace_id === $activeWorkspaceId));
  async function set(enabled: boolean) { busy = true; error = ""; try { applyTabsSnapshot(await setSplitView(enabled ? left : null, enabled ? right : null)); } catch (e) { error = describeError(e); } finally { busy = false; } }
</script>
<article class="tool-card">
  <strong>{$t("tools.split_view_915c")}</strong><p>{$t("tools.choose_two_tabs_from_this_workspace_click_a_page_to_focus_its_navigation_98b7")}</p>
  <div class="tool-actions">
    <select aria-label={$t("tools.left_split_tab_8f65")} bind:value={left}><option value="">{$t("tools.left_page_18fb")}</option>{#each candidates as tab (tab.id)}<option value={tab.id}>{tab.title}</option>{/each}</select>
    <select aria-label={$t("tools.right_split_tab_31be")} bind:value={right}><option value="">{$t("tools.right_page_6229")}</option>{#each candidates as tab (tab.id)}<option value={tab.id}>{tab.title}</option>{/each}</select>
    <button disabled={busy || !left || !right || left === right} onclick={() => set(true)}>{$t("tools.split_selected_tabs_be68")}</button>
    {#if $splitLayout}<button disabled={busy} onclick={() => set(false)}>{$t("tools.exit_split_view_693d")}</button>{/if}
  </div>
  {#if error}<ErrorMessage error={error} />{/if}
</article>
