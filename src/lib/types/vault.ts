export interface VaultStatus {
  initialized: boolean;
  /** The vault opens with a master password. */
  has_password: boolean;
  /** The vault opens with Windows Hello. */
  has_hello: boolean;
  /** This computer offers Windows Hello (always false outside Windows). */
  hello_available: boolean;
  unlocked: boolean;
  entry_count: number;
  auto_lock_minutes: number;
  retry_after_seconds: number;
}

export interface VaultEntryData {
  title: string;
  url: string;
  username: string;
  password: string;
  notes: string;
}

export interface VaultSummary {
  id: string;
  title: string;
  url: string;
  username: string;
  updated_at: number;
}

export interface VaultRevealed extends VaultEntryData {
  id: string;
  updated_at: number;
}

export interface VaultListing {
  entries: VaultSummary[];
  unreadable: number;
}

export const VAULT_AUTO_LOCK_MINUTES = [1, 5, 15, 30, 60] as const;
