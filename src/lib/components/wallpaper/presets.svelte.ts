import { logIgnored } from "../../services/ignored-error";
import { isLook, DEFAULT_LOOK } from "../../wallpaper/looks";
import { sanitizeParams, type EditorParams } from "../../wallpaper/params";
import { editor } from "./editorState.svelte";

export interface Preset { id: string; name: string; look: string; params: EditorParams }

const KEY = "spark.wallpaperPresets.v1";
const MAX_PRESETS = 100;
const MAX_NAME = 40;

function load(): Preset[] {
  try {
    const raw = JSON.parse(localStorage.getItem(KEY) ?? "null");
    if (!raw || !Array.isArray(raw.items)) return [];
    return raw.items.slice(0, MAX_PRESETS).flatMap((item: unknown): Preset[] => {
      const entry = item as Partial<Preset> | null;
      if (!entry || typeof entry.id !== "string" || typeof entry.name !== "string") return [];
      return [{ id: entry.id.slice(0, 64), name: entry.name.slice(0, MAX_NAME), look: isLook(entry.look) ? entry.look : DEFAULT_LOOK, params: sanitizeParams(entry.params) }];
    });
  } catch (error) { logIgnored(error); return []; }
}

/** The presets the user saved: a name plus a copy of every setting. They stay on this device, in the page's
 *  own storage. */
class Presets {
  items = $state<Preset[]>(load());

  private persist() {
    try { localStorage.setItem(KEY, JSON.stringify({ version: 1, items: $state.snapshot(this.items) })); }
    catch (error) { logIgnored(error); }
  }
  /** Saves the settings shown now under `name` (a blank name becomes "Preset N"). */
  save(name: string): Preset | null {
    if (this.items.length >= MAX_PRESETS) return null;
    const preset: Preset = {
      id: crypto.randomUUID(),
      name: name.trim().slice(0, MAX_NAME) || `Preset ${this.items.length + 1}`,
      look: editor.look,
      params: sanitizeParams($state.snapshot(editor.params)),
    };
    this.items = [...this.items, preset];
    this.persist();
    return preset;
  }
  /** Shows a saved preset in the editor. */
  apply(id: string) {
    const preset = this.items.find(item => item.id === id);
    if (!preset) return;
    editor.look = isLook(preset.look) ? preset.look : DEFAULT_LOOK;
    editor.params = sanitizeParams(preset.params);
  }
  remove(id: string) {
    this.items = this.items.filter(item => item.id !== id);
    this.persist();
  }
}

export const presets = new Presets();
