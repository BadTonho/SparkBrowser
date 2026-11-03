# Floating Tools panel

Tools opens 8 pixels below its toolbar button, with its right edge aligned to the button and at least 8 pixels of clearance from the window edges. The panel is at most 640 pixels wide and high; long content scrolls inside it. In compact split view, it opens below **More**. Requests from the Command Bar use the same anchor, and opening Tools from Zen restores the toolbar first.

The five tabs keep tab diagnostics, named sessions, tab organization, page notes and recently closed tabs together. Use Left/Right arrows or Home/End on the tabs, and Tab to move among controls. The selected tab has a subtle accent, with readable text, compact fields and a close icon.

Click outside, click the trigger again, press Esc or use the close icon to dismiss. Outside clicks only dismiss the panel. Escape and the close icon return focus to the trigger. The position follows window resizing and toolbar reordering. Removing the anchor, opening another browser panel or switching away from the application closes Tools.

## Native presentation and data ownership

On desktop, a dedicated transparent WebView presents the panel above both native page panes. Tools does not hide, reload or replace the pages. Existing browser panels keep their own behavior.

The shell retains the Svelte tool controllers, data operations, confirmations and the single queued note writer. It projects a structured, allowlisted presentation tree with opaque control handles to a small Svelte surface, using the same markup and styles as the web preview. The tree carries text and safe attributes, never executable HTML. The surface sends only typed control actions over a per-session authenticated bridge; it cannot invoke application commands. Stale sequences, unknown action shapes and invalid tokens are rejected. Unchanged presentations are not resent, and edits pending acknowledgement are not replaced by older snapshots.

Notes retain their existing debounce, error recovery and protected drafts when the panel closes. The panel does not introduce a second note writer or another storage format.

## Validation

Run `npm run test:native -- --case tools-popover` after the isolated native build. The scenario checks anchoring, compact mode, all five sections, native session and note edits, caret preservation, forged actions, direct IPC denial and preservation of both split pages. See [native integration](../contributing/native-tests.md).
