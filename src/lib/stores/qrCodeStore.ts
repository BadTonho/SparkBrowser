import { writable } from "svelte/store";

export const isQrCodeOpen = writable(false);
export const openQrCode = () => isQrCodeOpen.set(true);
export const closeQrCode = () => isQrCodeOpen.set(false);
