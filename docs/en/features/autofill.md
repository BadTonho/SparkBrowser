# Autofill

[Português](../../pt-BR/features/autofill.md)

Open **Settings → Autofill & Passwords**. The section opens on a list, like other browsers: **Password manager** (the [vault of logins](./password-manager.md)), **Payments** and **Contact info**.

Everything here lives in the same encrypted vault as the passwords. You create and unlock it once; [password-manager.md](./password-manager.md) explains how.

## Payments

Save cards to fill in at checkout. A card has a number, the name on it, month and year of expiry and an optional nickname.

- **The security code (CVV/CVC) is never saved.** You type it on the site each time.
- The list shows only the nickname (or the brand) and the **last four digits**, never the whole number.
- **Opening a card asks who you are**, every time: your master password or Windows Hello, like opening a password. The backend checks the proof. A card left open for five minutes without using the window closes itself.
- The number must be 12 to 19 digits and pass the card checksum (Luhn); spaces and dashes are ignored. Month and year are required.
- Up to 200 cards.

## Contact info

Save names, emails, phones and addresses. Every field is optional but at least one must be filled, and an email must look like one. Contact details open with the vault (no extra confirmation), because they are far less sensitive than a card or a password. Up to 200.

## Not included

- **Email aliases** (addresses that forward to your email) need an online service Spark does not run, so they are not offered.
- Autofill in pages and the question "save this card/address?" arrive in later versions of this feature; see the [changelog](../../../changelog/README.md).

## Private windows

The vault is not available in [private windows](./private-window.md).

## Storage

Cards and contact details are stored in the `vault_records` table, encrypted one by one with the vault key (AES-GCM, as logins). Like the vault, they are not part of backups, WebDAV sync or Drive, and not copied to private-window profiles. Details: [password manager](./password-manager.md) and the [security review](../security/account-containers-vault-review.md).
