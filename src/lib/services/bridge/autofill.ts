import { invoke } from "./runtime";
import type { AutofillKind, AutofillSettings, AutofillListing, AutofillRecord, AutofillRevealed, AutofillSummary } from "../../types/autofill";

export const listAutofillRecords = (kind: AutofillKind) => invoke<AutofillListing>("vault_records_list", { kind });
/** A card asks the person to prove who they are: with `currentPassword`, or with Windows Hello when it is null. */
export const revealAutofillRecord = (id: string, currentPassword: string | null) => invoke<AutofillRevealed>("vault_record_reveal", { id, currentPassword });
export const saveAutofillRecord = (id: string | null, record: AutofillRecord) => invoke<AutofillSummary>("vault_record_save", { id, record });
export const deleteAutofillRecord = (id: string) => invoke<void>("vault_record_delete", { id });

export const getAutofillSettings = () => invoke<AutofillSettings>("get_autofill_settings");
export const setAutofillSettings = (enabled: boolean, inPrivate: boolean) => invoke<AutofillSettings>("set_autofill_settings", { enabled, inPrivate });
