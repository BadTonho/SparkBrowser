import { lutFromParams } from "../../wallpaper/gradient";
import { defaultParams, resetEffects, resetGradient, type EditorParams } from "../../wallpaper/params";

/** What the editor is showing: the chosen look and its settings. Kept in memory only for now. */
class EditorState {
  look = $state("nebula");
  params = $state<EditorParams>(defaultParams());
  /** The gradient as a 256 x 1 strip, rebuilt whenever the colors, stops or blend change. */
  lut = $derived.by(() => lutFromParams(this.params));
  resetGradient() { this.params = resetGradient(this.params); }
  resetEffects() { this.params = resetEffects(this.params); }
}

export const editor = new EditorState();
