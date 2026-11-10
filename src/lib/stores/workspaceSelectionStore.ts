import { writable, readonly } from "svelte/store";
const selection = writable("ws-personal");
export const activeWorkspaceId = readonly(selection);
let revision = 0;
export const workspaceSelectionRevision = () => revision;
export function confirmWorkspaceSelection(id: string): void { revision++; selection.set(id); }
