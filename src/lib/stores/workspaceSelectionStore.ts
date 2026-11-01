import { writable } from "svelte/store";

// Shared independently of browserStore/workspaceStore to avoid a module cycle.
export const activeWorkspaceId = writable("ws-personal");
