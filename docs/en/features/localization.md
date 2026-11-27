# Interface languages

Settings → Appearance & Layout → Interface language offers System, Português and English. System uses Portuguese when the first preferred system language starts with `pt`, and English otherwise. A saved explicit choice takes priority. Changes apply immediately, including the document `lang`, controls, Command Bar, dates, numbers, plural forms and native overlay captions.

Page content, titles, URLs, notes, release notes and names supplied by the user are preserved. Operating system dialogs follow the system language. The language preference stays local to the browser profile.

English reference catalogs and Portuguese translations live in `src/lib/i18n/catalogs/`, split by interface domain. `MessageKey` is inferred from English keys. The English catalog is bundled with every interface window (it is the fallback and the source of `labelText`); the Portuguese one is joined in `src/lib/i18n/catalogs-pt.ts`, a separate file that a window reads only when Portuguese is the interface language. `main.ts` waits for it before mounting, so a window never opens in the wrong language. Register a new Portuguese domain file there, and the English one in `catalogs.ts`. Node tests require matching domain files, keys and interpolation parameters; Svelte checks validate references. Add a translation in both catalogs when introducing interface text. Use explicit message parameters instead of translating user values.

Errors use stable codes independently from their diagnostic text. Native bridge failures carry the command code; local validations can provide a specific `UserError` code. User messages use the catalog; expandable technical details retain the original diagnostic. Do not translate errors by matching English strings.

Run `npm test`, `npm run check` and `npm run test:native -- --case localization`. The native scenario changes languages through the actual settings control, checks Tools captions, restarts with the same isolated profile and verifies that tabs keep their identities and URLs.
