# Notes and sessions in the Command Bar

Typing searches saved notes by title, URL and text and sessions by name and page titles/URLs after 180 ms without typing. Matching ignores case and accents and treats special characters literally. Results show short text previews and counts, with explicit note and session badges. No full note or session document travels in search results.

Each category returns at most two matches. Its count describes the displayed matches; `2+` means more results exist, without computing an exact total. Transactional trigram indexes serve queries of three or more characters. Shorter queries and queries containing NUL scan normalized fields and keep the same literal substring behavior.

The list contains at most twelve items: at most three tabs and two of each other category. A nonempty query reserves two navigation actions. New query replies cannot replace newer results. Keyboard or pointer selection remains associated with its item while asynchronous results arrive.

Selecting a note opens its editor in Tools, bound to the original URL and preserving any draft. **Open page** is a separate action; **Follow the active page** returns to following navigation. Content remains plain text. Selecting a session opens its page list with pin indicators. **Restore** explicitly adds tabs to the current workspace. Removed items are rejected; a changed session must be reviewed again before restoration.

All queries and data stay local. Commands are restricted to the local `main` shell. Browser preview returns no persistent note/session search results. Tests cover bounded Unicode searches, delayed replies, category limits, original note URLs, removed items and reviewed session changes. Run `node tools/testing/native-smoke.mjs --case command-search` with the isolated native build.
