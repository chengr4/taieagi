/**
 * Minimal representation of an editable target element for cross-environment compatibility.
 */
export interface EditableLike {
  tagName?: string;
  type?: string;
  isContentEditable?: boolean;
  disabled?: boolean;
  readOnly?: boolean;
}

const SUPPORTED_INPUT_TYPES = new Set([
  "text",
  "search",
  "url",
  "tel",
  "email",
  "password",
  "",
]);

/**
 * Checks whether an event target is an interactive and editable text input element.
 */
export function isEditableTarget(target: unknown): boolean {
  if (!target || typeof target !== "object") {
    return false;
  }

  const el = target as EditableLike;

  if (el.disabled || el.readOnly) {
    return false;
  }

  if (el.isContentEditable === true) {
    return true;
  }

  const tagName = el.tagName?.toUpperCase();
  if (tagName === "TEXTAREA") {
    return true;
  }

  if (tagName === "INPUT") {
    const inputType = (el.type ?? "text").toLowerCase();
    return SUPPORTED_INPUT_TYPES.has(inputType);
  }

  return false;
}

export interface KeyboardEventLike {
  altKey: boolean;
  code?: string;
  key?: string;
}

/**
 * Checks whether a keyboard event matches the activation shortcut (Alt+T or macOS Option+T).
 */
export function isTriggerShortcut(event: KeyboardEventLike): boolean {
  if (!event.altKey) {
    return false;
  }

  // Windows / Linux / generic code match
  if (event.code === "KeyT") {
    return true;
  }

  // Fallback key match (macOS Option+T often produces '†')
  if (event.key === "t" || event.key === "T" || event.key === "†") {
    return true;
  }

  return false;
}

/**
 * Manages tracking of the currently focused editable element.
 */
export class FocusTracker {
  private currentEditable: unknown = null;

  public onFocus(target: unknown): void {
    if (isEditableTarget(target)) {
      this.currentEditable = target;
    }
  }

  public onBlur(target: unknown): void {
    if (this.currentEditable === target) {
      this.currentEditable = null;
    }
  }

  public getCurrent(): unknown {
    return this.currentEditable;
  }
}
