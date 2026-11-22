/** How a profile can be opened. */
export type LockMethod = "password" | "hello" | "vault";

export interface LockState {
  /** The profile is locked right now: nothing of it is shown until it is opened. */
  locked: boolean;
  /** The profile has an app lock. */
  enabled: boolean;
  /** Which ways in are turned on. */
  password: boolean;
  hello: boolean;
  vault: boolean;
  /** The profile's password vault has a master password, so it could be one of the ways in. */
  vault_available: boolean;
  /** This computer offers Windows Hello (always false outside Windows). */
  hello_available: boolean;
  /** Seconds to wait before another attempt is accepted (0 when none). */
  retry_after_seconds: number;
}

/** What the person presents to prove it is them. For Windows Hello `secret` is omitted: Windows asks. */
export interface LockProof {
  method: LockMethod;
  secret?: string;
}
