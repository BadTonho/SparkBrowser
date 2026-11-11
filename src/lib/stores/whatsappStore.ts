import { writable } from "svelte/store";
import type { WhatsAppDisplayMode } from "./settingsStore";

export const isWhatsAppWindowOpen = writable(false);
export const activeWhatsAppDisplayMode = writable<WhatsAppDisplayMode>("popover");
