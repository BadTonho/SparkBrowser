# Spark 0.1.6

## New features

- New **Download video** toolbar button (off by default; turn it on in the quick settings). It lists the videos of the loaded page, each with a small preview, and saves direct video files and unprotected HLS streams. YouTube, protected (DRM or encrypted) videos and live streams are not supported. The search has a time limit, so a page with slow streams or broken posters cannot keep the panel waiting, and a website cannot use it to make Spark send requests to your computer or local network.
- New buttons to open and close split view from the vertical tab list. The button at the top of the list opens a split with the tab you are viewing on the left and the most recently used tab on the right (a new blank tab if it is the only one). The button on the line between the panes closes the split. With horizontal tabs, a button in the left corner of the address row opens and closes the split, and **Alt+S** does the same in either layout.
- The search field of the new tab page now shows suggestions, like the address bar: your recent sites when you click it and, as you type, bookmarks, history and search engine suggestions (these follow the same **Search suggestions** setting and are never used in a private window).

## Improvements

- The volume popover is now simpler: it shows only the volume slider and its percentage, without the title, messages or list of tabs with media.
- Extensions installed from the Chrome Web Store or from a `.crx` file are now checked like Chrome does: the package must be signed by its developer and, from the store, be the extension you asked for. An altered or wrong package is refused and the version you already had stays installed. Old CRX2 files are no longer accepted.
- The **Local diagnostics** section of the settings now shows the problems Spark recorded: the latest entries of the local error log (Critical, Error, Warning and Info), newest first, with a filter by level, a button to copy them and the path of the log file. The internal performance counters moved to a collapsed "Internal performance" block.
- Address bar suggestions from history and bookmarks no longer show the page address under the title.
- The Glass window background style now has two types: **Classic** (the current translucent look) and **Liquid glass** (a clearer surface with a strong blur, boosted color and bright, light-catching edges). Pick one in Settings → Appearance & Layout after choosing Glass.
- The header of the vertical tab list is simpler: it reads only "Tabs (n)" and its buttons (split view, ephemeral tab, new tab) are now the same size and show only an icon; hover one to see its name.
- With vertical tabs and split view on, you can now drag a tab from one side of the split to the other in the tab list. Drop it anywhere on the other side, and the list shows where it will land while you drag. You can also drop it on the other pane's page.
- With vertical tabs and split view on, the tab list now has a line separating the tabs of each side of the split (right side and tabs outside the split), so you can tell which tabs belong to which side.
- The error log now says how many times an identical entry was repeated and skipped, shown on the next entry written for it.
- Redesigned the About & Updates settings section with a status summary card, clearer update progress and notices, and side-by-side cards for the browser repository and WebView2 runtime.
- Pages load with less work from the privacy shield: the style sheet that hides ads with the generic rules of the filter lists is now prepared once when the lists load, instead of being rebuilt for every page.
- History suggestions in the address bar and on the new tab page show more different sites: a site you visit often no longer takes several of the rows the list reads, and how often you visit it now counts in full when the list is ordered.
- Spark uses less power while idle: the audio background analyzer no longer wakes up five times a second when it is off or no new tab is open.
- The settings window opens with much less to load: each section is now read the first time you open it, instead of all of them at once.
- **Settings → Appearance & Layout** is now split into categories (Language and windows, Theme and colors, Window background, Layout, Toolbar and content), so each option is easier to find.
- **Settings → Privacy & Security** is now split into categories (Network privacy, Local history, Safe browsing, Privacy shield, Tracking), every option is a card with the same switches as the rest of the settings, and excluded sites are added inside their option's card and shown as removable chips.
- **Settings → Profiles** makes it easier to tell profiles apart: each one has labels (**In use**, **Main**, **Opens at start-up**, **App lock**) and its creation date, the profile in use is highlighted and no longer shows **Open**, and the page is split into categories (Your profiles, When Spark starts, and the security of the profile in use).

## Fixes

- Updated a component that could make Spark crash on a specially crafted page address.
- Saved passwords are no longer offered on other people's sites that only share an address ending with yours, such as two different `github.io` or `co.uk` sites. Passwords are still offered on the subdomains of your own site.
- Fixed a security problem where a malicious website could read the secret Spark uses to talk to its own scripts on that page, and use it to request your saved cards, contact details and passwords or to trigger browser shortcuts without you doing anything. The scripts of each feature now have separate secrets, and their messages can no longer be seen by the page.
- The error log no longer fills with entries for things that are normal: a page that has no media, and the settings tab that is still loading when it is opened. Real failures are still logged.
- The **Copy** button in Local diagnostics now works even when the panel does not have keyboard focus, and says so when the text could not be copied.
- In the quick settings, the selected option (such as the light or dark theme) is now clearly highlighted in both themes. Before, in the light theme the selected button looked fainter than the other one.
- The address bar suggestions are readable again with the Glass and Ice window styles: the list no longer lets the page behind it show through the rows.
- Address bar suggestions from your history and bookmarks now show each site's own icon, like the tabs do.
- Dropping the tab shown in one split pane onto the other pane's page no longer swaps the two panes. The tab just moves, and the pane it left shows another of its tabs.
- Spark no longer closes right after opening when the saved session has a tab it cannot restore (for example after going back to an older version). That tab is skipped and the problem is written to the error log.
- Errors that stop Spark from saving or closing (shown in the closing dialog) are now written to the error log with the exact failure, so they can be reported and fixed.
- An error inside a new tab widget (weather, Gmail, Spotify, YouTube) or the audio background no longer leaves the new tab page broken until Spark restarts: the widgets keep working and the problem is written to the error log. Saving the widget settings also no longer holds the widgets busy while the Spotify and audio settings are applied.
- Reader view messages (the author line and the "could not extract" error), the tip on the **Load .zip / .crx** extensions button and the screen reader labels of notifications now follow the interface language. Before, they were always in Portuguese. On Linux, Reader View now says it is available on Windows only, instead of showing an empty article with a made-up word count.
- Reader view now shows the article's images. Before, they appeared broken because the browser interface does not load images from the web; they now come from the copy the page already loaded, and images the page had not loaded are left out instead of showing as broken.
- When an installed extension cannot be loaded into a new tab on Windows, the reason is now written to the error log, with the extension that failed. Before, the failure left no trace.
- A problem inside Spark (an internal crash in one part of the browser) no longer makes other features stop working silently, or crash again each time they are used, until Spark restarts. The affected part recovers, keeps working, and the recovery is written to the error log with the place where it happened.
- The on/off options of **Toolbar and content** in the settings show their switches again, in cards like the other options. Before, they appeared as plain check boxes under the text.
