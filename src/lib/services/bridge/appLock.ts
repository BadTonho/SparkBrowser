import { invoke } from "./runtime";
import type { LockMethod, LockProof, LockState } from "../../types/appLock";

/** How the profile is locked and whether it is locked right now. It answers before start-up is ready. */
export const getLockState = () => invoke<LockState>("get_lock_state");
export const unlockWithPassword = (password: string) => invoke<void>("app_unlock_password", { password });
export const unlockWithVaultPassword = (password: string) => invoke<void>("app_unlock_vault_password", { password });
/** Windows asks for the PIN, fingerprint or face; nothing secret passes through the interface. */
export const unlockWithHello = () => invoke<void>("app_unlock_hello");
/** Quits from the lock screen. */
export const requestExit = () => invoke<void>("request_exit");
/** "Forgot the password": erases the data of this profile and starts it again, empty and unlocked. */
export const eraseLockedProfile = () => invoke<void>("erase_locked_profile", { confirm: true });

export const enableLock = (method: LockMethod, secret?: string) => invoke<void>("app_lock_enable", { method, secret: secret ?? null });
export const addLockMethod = (method: LockMethod, secret: string | undefined, proof: LockProof) => invoke<void>("app_lock_add_method", { method, secret: secret ?? null, proof: { method: proof.method, secret: proof.secret ?? null } });
export const removeLockMethod = (method: LockMethod, proof: LockProof) => invoke<void>("app_lock_remove_method", { method, proof: { method: proof.method, secret: proof.secret ?? null } });
export const disableLock = (proof: LockProof) => invoke<void>("app_lock_disable", { proof: { method: proof.method, secret: proof.secret ?? null } });
