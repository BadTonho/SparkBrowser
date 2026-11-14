<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../../i18n";
  import { currentUrl } from "../../stores/browserStore";
  import {
    installedExtensions,
    installFromWebStore,
    isExtensionsModalOpen,
    extensionsError,
  } from "../../stores/extensionsStore";
  import { showToast } from "../../stores/notificationStore";
  import { listen } from "@tauri-apps/api/event";
  import { isTauriRuntime } from "../../services/tauri-bridge";
  import type { ExtensionDetails } from "../../types/extensions";

  const WEBSTORE_REGEX =
    /(?:chromewebstore\.google\.com|chrome\.google\.com\/webstore)\/detail\/(?:[^\/]+\/)?([a-p]{32})/;

  const extensionId = $derived.by(() => {
    const match = $currentUrl ? $currentUrl.match(WEBSTORE_REGEX) : null;
    return match ? match[1] : null;
  });

  const isInstalled = $derived(
    Boolean(extensionId && $installedExtensions.some((ext) => ext.id === extensionId))
  );

  let installing = $state(false);

  async function handleInstall() {
    if (installing || !extensionId) return;
    if (isInstalled) {
      isExtensionsModalOpen.set(true);
      return;
    }

    installing = true;
    try {
      const result = await installFromWebStore(extensionId);
      if (result) {
        showToast({
          type: "success",
          message: $t("extensions.install_success", { name: result.name }),
          actionLabel: $t("extensions.manage"),
          onAction: () => {
            isExtensionsModalOpen.set(true);
          },
        });
      } else {
        showToast({
          type: "error",
          message: $extensionsError || $t("extensions.error_loading"),
        });
      }
    } catch (err: any) {
      showToast({
        type: "error",
        message: err?.message || String(err),
      });
    } finally {
      installing = false;
    }
  }

  onMount(() => {
    let unlistenInstalled: (() => void) | undefined;
    let unlistenFailed: (() => void) | undefined;

    if (isTauriRuntime()) {
      void listen<ExtensionDetails>("extension-installed", (event) => {
        const details = event.payload;
        installedExtensions.update((list) => {
          const filtered = list.filter((e) => e.id !== details.id);
          return [...filtered, details];
        });
        showToast({
          type: "success",
          message: $t("extensions.install_success", { name: details.name }),
          actionLabel: $t("extensions.manage"),
          onAction: () => {
            isExtensionsModalOpen.set(true);
          },
        });
      }).then((fn) => {
        unlistenInstalled = fn;
      });

      void listen<{ id: string; error: string }>("extension-install-failed", (event) => {
        showToast({
          type: "error",
          message: event.payload.error || $t("extensions.error_loading"),
        });
      }).then((fn) => {
        unlistenFailed = fn;
      });
    }

    return () => {
      unlistenInstalled?.();
      unlistenFailed?.();
    };
  });
</script>

{#if extensionId}
  <button
    type="button"
    class="webstore-install-btn"
    class:installed={isInstalled}
    class:installing
    disabled={installing}
    onclick={handleInstall}
    title={isInstalled
      ? $t("extensions.installed_badge")
      : $t("extensions.install_in_spark")}
    aria-label={isInstalled
      ? $t("extensions.installed_badge")
      : $t("extensions.install_in_spark")}
  >
    {#if installing}
      <span class="spinner" aria-hidden="true"></span>
      <span>{$t("extensions.installing")}</span>
    {:else if isInstalled}
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2.5"
        width="12"
        height="12"
        aria-hidden="true"
      >
        <polyline points="20 6 9 17 4 12" />
      </svg>
      <span>{$t("extensions.installed_badge")}</span>
    {:else}
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        width="12"
        height="12"
        aria-hidden="true"
      >
        <path
          d="M20.5 11H19V7c0-1.1-.9-2-2-2h-4V3.5a2.5 2.5 0 0 0-5 0V5H4c-1.1 0-1.99.9-1.99 2v3.8H3.5c1.49 0 2.7 1.21 2.7 2.7s-1.21 2.7-2.7 2.7H2V20c0 1.1.9 2 2 2h3.8v-1.5c0-1.49 1.21-2.7 2.7-2.7 1.49 0 2.7 1.21 2.7 2.7V22H17c1.1 0 2-.9 2-2v-4h1.5a2.5 2.5 0 0 0 0-5z"
        />
      </svg>
      <span>{$t("extensions.install_in_spark")}</span>
    {/if}
  </button>
{/if}

<style>
  .webstore-install-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 24px;
    padding: 2px 9px;
    border-radius: 5px;
    font-size: 11px;
    font-weight: 550;
    cursor: pointer;
    background: #2563eb;
    color: #ffffff;
    border: 1px solid rgba(255, 255, 255, 0.2);
    box-shadow: 0 1px 4px rgba(37, 99, 235, 0.35);
    transition: all 0.15s ease;
    flex-shrink: 0;
    user-select: none;
    margin-right: 4px;
  }

  .webstore-install-btn:hover:not(:disabled) {
    background: #1d4ed8;
    box-shadow: 0 2px 6px rgba(37, 99, 235, 0.5);
  }

  .webstore-install-btn.installed {
    background: rgba(52, 199, 89, 0.12);
    color: #34c759;
    border-color: rgba(52, 199, 89, 0.3);
    box-shadow: none;
  }

  .webstore-install-btn.installed:hover {
    background: rgba(52, 199, 89, 0.22);
  }

  .webstore-install-btn.installing {
    opacity: 0.85;
    cursor: wait;
  }

  .spinner {
    width: 10px;
    height: 10px;
    border: 1.5px solid rgba(255, 255, 255, 0.4);
    border-top-color: #ffffff;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>

