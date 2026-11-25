# Autofill

[Português](../../pt-BR/features/autofill.md)

Open **Settings → Autofill & Passwords**. The section opens on a list, like other browsers: **Password manager** (the [vault of logins](./password-manager.md)), **Payments** and **Contact info**.

Everything here lives in the same encrypted vault as the passwords. You create and unlock it once; [password-manager.md](./password-manager.md) explains how.

## Payments

Save cards to fill in at checkout (see [Filling in pages](#filling-in-pages)). A card has a number, the name on it, month and year of expiry and an optional nickname.

- **The security code (CVV/CVC) is never saved.** You type it on the site each time.
- The list shows only the nickname (or the brand) and the **last four digits**, never the whole number.
- **Opening a card asks who you are**, every time: your master password or Windows Hello, like opening a password. The backend checks the proof. A card left open for five minutes without using the window closes itself.
- The number must be 12 to 19 digits and pass the card checksum (Luhn); spaces and dashes are ignored. Month and year are required.
- Up to 200 cards.

## Contact info

Save names, emails, phones and addresses. Every field is optional but at least one must be filled, and an email must look like one. Contact details open with the vault (no extra confirmation), because they are far less sensitive than a card or a password. Up to 200.

## Filling in pages

Click a login, card or contact field and Spark lists what fits under it. Pick an item with the mouse (or ArrowDown and Enter) and the form fills in.

- **Logins** are offered on a page of the site they were saved for (the saved address or one of its subdomains), never on look-alike addresses, and never on the plain `http://` version of a site saved with `https://`. Click the username or password field.
- **Cards** are offered under card number, name and expiry fields. The security code is never filled: type it yourself. Month and year fill selects and `MM/YY` or `MM/YYYY` fields alike.
- **Contact info** is offered under name, email, phone and address fields.
- The list only shows titles (an email, `Visa •••• 4242`), never a password or a whole card number. It lives in a closed part of the page that the site cannot read, and only reacts to real clicks and keys. What fills the form comes from Spark after you pick, and Spark only fills an item that was in the list it just showed on that page. The script that shows the list keeps its secret and its handlers away from the site's own code, so a site cannot pick an item for you.
- With **Use Windows Hello when showing or copying passwords** on (see [password manager](./password-manager.md)), filling a login or a card asks for Windows Hello each time. Contact info never asks.
- **Locked vault:** the list says so. With Windows Hello it offers **Unlock with Windows Hello**; otherwise unlock it in Settings.
- Search boxes, other frames inside a page (such as payment iframes) and Spark's own pages are not filled. Only the page you are on is: if you move to another site before picking, nothing is filled.
- The list text follows the language of your system, not the language chosen in Spark.

### Saving what you type

Submit a sign-up, a login, a payment or an address form and Spark may ask, in a bar at the top right of the page: **Save the password for example.com?**, **Update the password for example.com?**, **Save the card Visa •••• 4242?** or **Save this contact info?**. Nothing is saved until you click **Save** (or **Update**).

- It asks only with the vault **unlocked**, never in a private window, and not when the same login or card is already saved. A changed password offers **Update**.
- A card is offered only with a valid number (checksum) and an expiry date; its security code is never read. Contact info needs an address or phone and at least three filled fields.
- If the submit sends you to another page of the same site, the question is asked there (for up to 90 seconds).
- **Not now** stops that same question for that site until Spark restarts.

### The switches

Under the list in Settings:

- **Offer to fill in forms** (on by default) turns all of this on or off.
- **Allow autofill in private windows** (off by default). When on, a **new** private window carries a **read-only copy** of the encrypted vault: you unlock it there (Windows Hello from the list, or the master password in its Settings) and can fill, but never add, edit or delete. The copy disappears with the window. Windows already open do not change.

## Not included

- **Email aliases** (addresses that forward to your email) need an online service Spark does not run, so they are not offered.

## Tests

```
node tools/testing/native-smoke.mjs --case autofill
node --test tests/autofill-roles.test.mjs
```

`autofill` fills a login, a card and a contact in a real page and checks the cases above (other site, search box, locked vault, switch off) and the save questions (save, already saved, Not now). `autofill-roles` covers how fields are recognised.

## Storage

Cards and contact details are stored in the `vault_records` table, encrypted one by one with the vault key (AES-GCM, as logins). Like the vault, they are not part of backups, WebDAV sync or Drive, and not copied to private-window profiles. Details: [password manager](./password-manager.md) and the [security review](../security/account-containers-vault-review.md).
