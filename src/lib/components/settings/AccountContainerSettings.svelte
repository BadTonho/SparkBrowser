<script lang="ts">
  import { onMount } from "svelte";
  import { locale } from "../../i18n";
  import { tabs } from "../../stores/browserStore";
  import { accountContainers, loadAccountContainers, createAccountContainer, renameAccountContainer, deleteAccountContainer, setTabAccountContainer, createContainerTab } from "../../stores/accountContainersStore";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  const messages = {
    en: { title: "Account containers", description: "Separate site sessions that follow this container across workspaces.", name: "Container name", create: "Create", rename: "Rename", open: "Open tab", use: "Use in active tab", remove: "Remove from active tab", delete: "Delete", confirm: "Delete this container and its site data? Its tabs will return to the default identity.", yes: "Delete permanently", cancel: "Cancel", changing: "Changing the identity reloads the active tab and discards its live page state.", apply: "Confirm identity change", pending: "Deletion pending; retry", unsupported: "Available in the desktop application." },
    "pt-BR": { title: "Contêineres de contas", description: "Sessões de sites separadas que acompanham o contêiner entre workspaces.", name: "Nome do contêiner", create: "Criar", rename: "Renomear", open: "Abrir aba", use: "Usar na aba ativa", remove: "Remover da aba ativa", delete: "Excluir", confirm: "Excluir este contêiner e os dados dos sites? Suas abas voltarão à identidade padrão.", yes: "Excluir permanentemente", cancel: "Cancelar", changing: "Trocar a identidade recarrega a aba ativa e descarta o estado atual da página.", apply: "Confirmar troca de identidade", pending: "Exclusão pendente; tente novamente", unsupported: "Disponível no aplicativo para desktop." },
  };
  let text = $derived($locale === "pt-BR" ? messages["pt-BR"] : messages.en);
  let name = $state("");
  let renameId = $state("");
  let pendingDelete = $state("");
  let pendingSwitch = $state<string | null | undefined>(undefined);
  let busy = $state(false);
  let error = $state("");
  let active = $derived($tabs.find(tab => tab.is_active));
  onMount(() => { void loadAccountContainers().catch(e => error = String(e)); });
  async function run(action: () => Promise<void>) {
    busy = true; error = "";
    try { await action(); } catch (e) { error = String(e); } finally { busy = false; }
  }
</script>
<section class="accounts" aria-labelledby="accounts-title">
  <h2 id="accounts-title">{text.title}</h2>
  <p>{text.description}</p>
  {#if !isTauriRuntime()}<p>{text.unsupported}</p>{:else}
    <label>{text.name}<input bind:value={name} maxlength="80" autocomplete="off" /></label>
    <button disabled={busy || !name.trim()} onclick={() => run(async () => { if (renameId) await renameAccountContainer(renameId, name); else await createAccountContainer(name); name = ""; renameId = ""; })}>{renameId ? text.rename : text.create}</button>
    {#if renameId}<button disabled={busy} onclick={() => { renameId = ""; name = ""; }}>{text.cancel}</button>{/if}
    {#each $accountContainers as account (account.id)}
      <div class="account-row">
        <strong>◉ {account.name}</strong>
        {#if account.deleting}<span>{text.pending}</span>{/if}
        <button disabled={busy || account.deleting} onclick={() => { name = account.name; renameId = account.id; }}>{text.rename}</button>
        <button disabled={busy || account.deleting} onclick={() => run(() => createContainerTab(account.id))}>{text.open}</button>
        <button disabled={busy || account.deleting || !active || active.is_ephemeral || active.container_id === account.id} onclick={() => pendingSwitch = account.id}>{text.use}</button>
        <button disabled={busy} onclick={() => pendingDelete = account.id}>{text.delete}</button>
      </div>
      {#if pendingDelete === account.id}
        <div class="confirm" role="group" aria-label={text.confirm}><p>{text.confirm}</p>
          <button disabled={busy} onclick={() => run(async () => { await deleteAccountContainer(account.id); pendingDelete = ""; })}>{text.yes}</button>
          <button disabled={busy} onclick={() => pendingDelete = ""}>{text.cancel}</button>
        </div>
      {/if}
    {/each}
    {#if active?.container_id}<button disabled={busy} onclick={() => pendingSwitch = null}>{text.remove}</button>{/if}
    {#if pendingSwitch !== undefined}
      <div class="confirm" role="group" aria-label={text.changing}><p>{text.changing}</p>
        <button disabled={busy || !active} onclick={() => run(async () => { if (active) await setTabAccountContainer(active.id, pendingSwitch ?? null); pendingSwitch = undefined; })}>{text.apply}</button>
        <button disabled={busy} onclick={() => pendingSwitch = undefined}>{text.cancel}</button>
      </div>
    {/if}
    {#if error}<p role="alert">{error}</p>{/if}
  {/if}
</section>
<style>
  .accounts { margin: 24px 0; padding: 16px; border: 1px solid var(--border-subtle); border-radius: 10px; }
  h2 { font-size: 16px; } p { color: var(--text-secondary); } label { display: grid; gap: 5px; margin: 10px 0; }
  input { background: var(--bg-glass); color: var(--text-primary); border: 1px solid var(--border-subtle); padding: 8px; }
  button { padding: 6px 9px; margin: 3px; color: var(--text-primary); background: var(--bg-glass-hover); border: 1px solid var(--border-subtle); border-radius: 5px; cursor: pointer; }
  button:disabled { opacity: .5; cursor: default; } .account-row { display: flex; flex-wrap: wrap; align-items: center; gap: 4px; margin-top: 8px; } strong { flex: 1 0 100px; } .confirm { border: 1px solid var(--border-subtle); padding: 8px; }
</style>
