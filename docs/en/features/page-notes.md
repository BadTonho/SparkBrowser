# Page notes

Open **Tools → Notes** to edit plain text for the active page. One note is shared across workspaces for each normalized HTTP/HTTPS URL. Fragments after `#` are ignored; paths and query parameters remain significant.

Changes save after 500 ms without typing. Drafts remain associated with their original URLs when switching pages or closing the panel. A single queue orders saves and deletions, and late replies cannot replace newer text. Status shows Unsaved, Saving, Saved or Error; Retry retains and resubmits a failed draft. Notes require the desktop runtime, are stored in the local application database and are limited to 200 KB each. Abrupt termination can lose changes not yet saved.

Search saved text, titles and URLs; results show up to 100 matches with previews of at most 180 characters. Opening a note loads its complete text. Refresh after editing to update that list. Open a note's page in a new tab or delete it after confirmation. Content is rendered as text, never as HTML.

The saved-note cache retains up to 32 old entries or 2,000,000 UTF-16 code units of saved text. Active editors and unsaved/saving/failed drafts are protected, so these limits may be exceeded while work remains pending. Eviction removes only the in-memory saved copy; reopening fetches the note from SQLite.
