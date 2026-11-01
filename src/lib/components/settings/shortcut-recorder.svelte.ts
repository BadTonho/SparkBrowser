import { get } from "svelte/store";
import {
  SHORTCUT_DEFINITIONS,
  recordingShortcutId,
  setShortcutBinding,
  type ShortcutCombo,
} from "../../stores/shortcutsStore";
export function createShortcutRecorder() {
  function processShortcutRecording(event: KeyboardEvent): boolean {
    const id = get(recordingShortcutId);
    if(!id) return false;

    event.preventDefault();
    event.stopPropagation();
    const definition = SHORTCUT_DEFINITIONS.find((item) => item.id === id);
    if(!definition) return true;

    if(event.key === "Escape") {
      recordingShortcutId.set(null);
      recordingError = "";
      return true;
    }

    const modifierKeys = ["Alt", "Control", "Shift", "Meta"];
    let binding: ShortcutCombo;
    if(definition.hold) {
      if(!modifierKeys.includes(event.key)) {
        recordingError = "Choose Alt, Ctrl, Shift, or Meta.";
        return true;
      }
      binding = { key: event.key, ctrl: false, alt: false, shift: false, meta: false };
    } else {
      if(modifierKeys.includes(event.key)) return true;
      const key = definition.sequence ? "Digit" : event.key.length === 1 ? event.key.toUpperCase() : event.key;
      binding = {
        key,
        ctrl: event.ctrlKey,
        alt: event.altKey,
        shift: event.shiftKey,
        meta: event.metaKey,
      };
    }

    const error = setShortcutBinding(id, binding);
    if(error) {
      recordingError = error;
    } else {
      recordingError = "";
      recordingShortcutId.set(null);
    }
    return true;
  }

  function processMouseShortcutRecording(event: MouseEvent): boolean {
    const id = get(recordingShortcutId);
    if(!id || (event.button !== 3 && event.button !== 4)) return false;
    event.preventDefault();
    event.stopPropagation();
    const definition = SHORTCUT_DEFINITIONS.find((item) => item.id === id);
    if(definition?.hold || definition?.sequence) {
      recordingError = definition.hold
        ? "Choose Alt, Ctrl, Shift, or Meta."
        : "This shortcut requires a number key.";
      return true;
    }
    const binding: ShortcutCombo = {
      key: event.button === 3 ? "Mouse4" : "Mouse5",
      ctrl: event.ctrlKey,
      alt: event.altKey,
      shift: event.shiftKey,
      meta: event.metaKey,
    };
    const error = setShortcutBinding(id, binding);
    if(error) {
      recordingError = error;
    } else {
      recordingError = "";
      recordingShortcutId.set(null);
    }
    return true;
  }

  let recordingError = $state("");

  function startRecording(id: string): void {
    recordingError = "";
    recordingShortcutId.set(id);
  }

  return { get error() { return recordingError; }, keyboard: processShortcutRecording, mouse: processMouseShortcutRecording, start: startRecording };
}
