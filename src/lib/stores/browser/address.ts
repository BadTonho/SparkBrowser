import { writable, readonly } from "svelte/store";
import { NEW_TAB_URL } from "../../services/new-tab";
const mutableUrl = writable<string>(NEW_TAB_URL);
export const currentUrl = readonly(mutableUrl);
export const addressInput = writable<string>("");
export const isAddressEditing = writable(false);
export const setConfirmedUrl = (url: string) => mutableUrl.set(url);
