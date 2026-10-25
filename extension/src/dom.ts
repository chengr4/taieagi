/**
 * Inserts text into an interactive editable target element (input, textarea, or contenteditable).
 * Prefers synthetic ClipboardEvent('paste') for contenteditable to avoid breaking rich-text
 * framework states (Draft.js on X, Lexical, Slate), then falls back to execCommand or DOM APIs.
 */
export function insertTextIntoTarget(target: HTMLElement, text: string): boolean {
  try {
    target.focus();
  } catch {
    // Focus may fail on detached or mocked nodes
  }

  // Strategy 1: For contenteditable elements, try synthetic paste with DataTransfer.
  // Modern rich-text frameworks (e.g. Draft.js on X, Lexical, React) intercept paste,
  // call preventDefault(), and update their internal state cleanly without text duplication.
  if (target.isContentEditable) {
    if (typeof ClipboardEvent !== "undefined" && typeof DataTransfer !== "undefined") {
      try {
        const dt = new DataTransfer();
        dt.setData("text/plain", text);
        const pasteEvent = new ClipboardEvent("paste", {
          bubbles: true,
          cancelable: true,
          clipboardData: dt,
        });
        target.dispatchEvent(pasteEvent);
        if (pasteEvent.defaultPrevented) {
          return true;
        }
      } catch {
        // Fallback to execCommand / Range below
      }
    }
  }

  // Strategy 2: Attempt standard execCommand in browser environment
  if (typeof document !== "undefined" && typeof document.execCommand === "function") {
    try {
      const success = document.execCommand("insertText", false, text);
      if (success) {
        return true;
      }
    } catch {
      // Fallback below if execCommand fails
    }
  }

  // Fallback for standard input and textarea elements
  const inputEl = target as HTMLInputElement | HTMLTextAreaElement;
  if (
    typeof inputEl.value === "string" &&
    typeof inputEl.selectionStart === "number" &&
    typeof inputEl.selectionEnd === "number"
  ) {
    const start = inputEl.selectionStart;
    const end = inputEl.selectionEnd;
    const original = inputEl.value;
    inputEl.value = original.slice(0, start) + text + original.slice(end);
    inputEl.selectionStart = start + text.length;
    inputEl.selectionEnd = start + text.length;

    try {
      inputEl.dispatchEvent(new Event("input", { bubbles: true }));
      inputEl.dispatchEvent(new Event("change", { bubbles: true }));
    } catch {
      // Event dispatch may be limited in test environments
    }
    return true;
  }

  // Fallback for contenteditable elements
  if (typeof window !== "undefined" && window.getSelection) {
    const sel = window.getSelection();
    if (sel && sel.rangeCount > 0) {
      const range = sel.getRangeAt(0);
      range.deleteContents();
      const textNode = document.createTextNode(text);
      range.insertNode(textNode);
      range.setStartAfter(textNode);
      range.setEndAfter(textNode);
      sel.removeAllRanges();
      sel.addRange(range);
      target.dispatchEvent(new Event("input", { bubbles: true }));
      return true;
    }
  }

  return false;
}
