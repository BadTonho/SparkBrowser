import { invoke } from "./runtime";
import type { VaultEntryData, VaultListing, VaultRevealed, VaultStatus, VaultSummary } from "../../types/vault";

export const getVaultStatus = () => invoke<VaultStatus>("vault_status");
/** Resolves with the recovery key, which the backend shows once and never stores. */
export const createVault = (masterPassword: string | null, useHello: boolean) => invoke<string>("vault_create", { masterPassword, useHello });
export const unlockVault = (password: string) => invoke<void>("vault_unlock", { password });
export const unlockVaultWithRecovery = (recoveryKey: string) => invoke<void>("vault_unlock_recovery", { recoveryKey });
/** Windows asks for the PIN, fingerprint or face; nothing secret passes through the interface. */
export const unlockVaultWithHello = () => invoke<void>("vault_unlock_hello");
export const enableVaultHello = () => invoke<void>("vault_hello_enable");
export const disableVaultHello = () => invoke<void>("vault_hello_disable");
export const enableVaultPassword = (newPassword: string) => invoke<void>("vault_password_enable", { newPassword });
export const disableVaultPassword = (current: string) => invoke<void>("vault_password_disable", { current });
export const lockVault = () => invoke<void>("vault_lock");
export const listVaultEntries = () => invoke<VaultListing>("vault_list");
export const revealVaultEntry = (id: string) => invoke<VaultRevealed>("vault_reveal", { id });
export const saveVaultEntry = (id: string | null, entry: VaultEntryData) => invoke<VaultSummary>("vault_save_entry", { id, entry });
export const deleteVaultEntry = (id: string) => invoke<void>("vault_delete_entry", { id });
export const changeVaultPassword = (current: string, newPassword: string) => invoke<void>("vault_change_master_password", { current, newPassword });
export const setVaultAutoLock = (minutes: number) => invoke<void>("vault_set_auto_lock", { minutes });
/** The backend copies the password itself and clears the clipboard again after 30 seconds. */
export const copyVaultPassword = (id: string) => invoke<void>("vault_copy_password", { id });
export const copyVaultText = (text: string) => invoke<void>("vault_copy_text", { text });
export const resetVault = () => invoke<void>("vault_reset", { confirm: true });
