<script lang="ts">
  import PanelNodeView from "./PanelViewNode.svelte";
  import type { PanelViewNode } from "../../services/panel-presentation";
  let { node, pending, ack }: { node: PanelViewNode; pending: Map<number, number>; ack: number } = $props();
  function field(element: HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement, value: PanelViewNode) {
    function sync(next: PanelViewNode) {
      if ((pending.get(next.id) ?? 0) > ack) return;
      if (next.value !== undefined && element.value !== next.value) element.value = next.value;
      if (element instanceof HTMLInputElement && next.checked !== undefined) element.checked = next.checked;
    }
    sync(value);
    return { update: sync };
  }
</script>

{#if !node.tag}
  {node.text ?? ""}
{:else if node.tag === "input"}
  <input {...node.attributes} data-panel-control={node.id} use:field={node} />
{:else if node.tag === "textarea"}
  <textarea {...node.attributes} data-panel-control={node.id} use:field={node}></textarea>
{:else if node.tag === "select"}
  <select {...node.attributes} data-panel-control={node.id} use:field={node}>
    {#each node.children ?? [] as child (child.id)}<PanelNodeView node={child} {pending} {ack} />{/each}
  </select>
{:else}
  <svelte:element this={node.tag} {...node.attributes} data-panel-control={node.id}
    xmlns={["svg", "path", "circle"].includes(node.tag) ? "http://www.w3.org/2000/svg" : undefined}>
    {#each node.children ?? [] as child (child.id)}<PanelNodeView node={child} {pending} {ack} />{/each}
  </svelte:element>
{/if}
