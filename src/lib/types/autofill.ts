/** A payment card as the editor sees it. The security code (CVV) is never stored. */
export interface AutofillCard {
  kind: "card";
  nickname: string;
  holder: string;
  /** Digits only once saved; the person may type spaces. */
  number: string;
  exp_month: number;
  exp_year: number;
}

export interface AutofillContact {
  kind: "contact";
  name: string;
  organization: string;
  email: string;
  phone: string;
  street: string;
  street2: string;
  city: string;
  region: string;
  postal_code: string;
  country: string;
}

export type AutofillRecord = AutofillCard | AutofillContact;
export type AutofillKind = AutofillRecord["kind"];

/** A row of the list: a card shows only its last four digits. */
export interface AutofillSummary {
  id: string;
  kind: AutofillKind;
  title: string;
  subtitle: string;
  updated_at: number;
}

export interface AutofillListing {
  records: AutofillSummary[];
  unreadable: number;
}

/** The two switches of Settings → Autofill & passwords. */
export interface AutofillSettings {
  /** Offer to fill in pages. */
  enabled: boolean;
  /** Also offer it in private windows, which then carry a read-only copy of the vault. */
  in_private: boolean;
}

/** The pages of Settings → Autofill and passwords. */
export type AutofillPage = "home" | "passwords" | "payments" | "contacts";

export type AutofillRevealed = AutofillRecord & { id: string; updated_at: number };
