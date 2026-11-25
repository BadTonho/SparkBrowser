/**
 * Copies text to the clipboard. The asynchronous clipboard needs the document to have focus, which a panel
 * that is drawn by the shell does not always have, so a hidden text field and the copy command are the fallback.
 * Returns whether the text was copied.
 */
export async function copyText(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    // Falls through to the copy command.
  }
  const field = document.createElement("textarea");
  field.value = text;
  field.setAttribute("readonly", "");
  field.style.cssText = "position:fixed;top:0;left:-9999px;opacity:0";
  document.body.appendChild(field);
  try {
    field.select();
    return document.execCommand("copy");
  } catch {
    return false;
  } finally {
    field.remove();
  }
}
