import { writable } from "svelte/store";
import { deleteAutofillRecord, getAutofillSettings, listAutofillRecords, revealAutofillRecord, saveAutofillRecord, setAutofillSettings } from "../services/bridge/autofill";
import type { AutofillKind, AutofillRecord, AutofillRevealed, AutofillSettings, AutofillSummary } from "../types/autofill";
import { runVaultCall, syncVaultLock } from "./vaultStore";

/** The saved cards and contact details, as list rows (a card shows only its last four digits). */
export const autofillCards = writable<AutofillSummary[]>([]);
export const autofillContacts = writable<AutofillSummary[]>([]);
export const autofillUnreadable = writable(0);

const lists = { card: autofillCards, contact: autofillContacts } as const;

export async function loadAutofill(kind: AutofillKind): Promise<void> {
  const listing = await runVaultCall(() => listAutofillRecords(kind), true);
  if (!listing) {
    await syncVaultLock();
    return;
  }
  lists[kind].set(listing.records);
  autofillUnreadable.set(listing.unreadable);
}

/** Opens a record in full. A card first asks who is there: the master password, or Windows Hello when null. */
export async function revealAutofill(id: string, currentPassword: string | null): Promise<AutofillRevealed | undefined> {
  const record = await runVaultCall(() => revealAutofillRecord(id, currentPassword));
  if (!record) await syncVaultLock();
  return record;
}

export async function saveAutofill(id: string | null, record: AutofillRecord): Promise<boolean> {
  const saved = await runVaultCall(() => saveAutofillRecord(id, record));
  if (!saved) {
    await syncVaultLock();
    return false;
  }
  await loadAutofill(record.kind);
  return true;
}

export async function removeAutofill(id: string, kind: AutofillKind): Promise<void> {
  const done = await runVaultCall(async () => { await deleteAutofillRecord(id); return true; });
  if (done) await loadAutofill(kind);
  else await syncVaultLock();
}

export function forgetAutofill(): void {
  autofillCards.set([]);
  autofillContacts.set([]);
  autofillUnreadable.set(0);
}

/** The two switches (null until the backend answered). */
export const autofillSettings = writable<AutofillSettings | null>(null);

export async function loadAutofillSettings(): Promise<void> {
  const settings = await runVaultCall(getAutofillSettings, true);
  if (settings) autofillSettings.set(settings);
}

export async function changeAutofillSettings(next: AutofillSettings): Promise<void> {
  const saved = await runVaultCall(() => setAutofillSettings(next.enabled, next.in_private));
  if (saved) autofillSettings.set(saved);
}
