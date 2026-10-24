import { describe, expect, it } from "bun:test";
import { FocusTracker, isEditableTarget, isTriggerShortcut } from "./focus";

describe("isEditableTarget", () => {
  it("recognizes standard textarea", () => {
    expect(isEditableTarget({ tagName: "TEXTAREA" })).toBe(true);
    expect(isEditableTarget({ tagName: "textarea" })).toBe(true);
  });

  it("recognizes text input types", () => {
    expect(isEditableTarget({ tagName: "INPUT", type: "text" })).toBe(true);
    expect(isEditableTarget({ tagName: "INPUT", type: "search" })).toBe(true);
    expect(isEditableTarget({ tagName: "INPUT", type: "email" })).toBe(true);
    expect(isEditableTarget({ tagName: "INPUT", type: "url" })).toBe(true);
    expect(isEditableTarget({ tagName: "INPUT" })).toBe(true); // default type is text
  });

  it("rejects non-text input types", () => {
    expect(isEditableTarget({ tagName: "INPUT", type: "checkbox" })).toBe(false);
    expect(isEditableTarget({ tagName: "INPUT", type: "radio" })).toBe(false);
    expect(isEditableTarget({ tagName: "INPUT", type: "button" })).toBe(false);
    expect(isEditableTarget({ tagName: "INPUT", type: "submit" })).toBe(false);
    expect(isEditableTarget({ tagName: "INPUT", type: "file" })).toBe(false);
  });

  it("rejects disabled or readonly inputs", () => {
    expect(isEditableTarget({ tagName: "INPUT", type: "text", disabled: true })).toBe(false);
    expect(isEditableTarget({ tagName: "INPUT", type: "text", readOnly: true })).toBe(false);
    expect(isEditableTarget({ tagName: "TEXTAREA", disabled: true })).toBe(false);
    expect(isEditableTarget({ tagName: "TEXTAREA", readOnly: true })).toBe(false);
  });

  it("recognizes contenteditable elements", () => {
    expect(isEditableTarget({ tagName: "DIV", isContentEditable: true })).toBe(true);
    expect(isEditableTarget({ tagName: "SPAN", isContentEditable: true })).toBe(true);
    expect(isEditableTarget({ tagName: "DIV", isContentEditable: false })).toBe(false);
  });

  it("rejects non-editable standard elements and nulls", () => {
    expect(isEditableTarget({ tagName: "DIV" })).toBe(false);
    expect(isEditableTarget(null)).toBe(false);
    expect(isEditableTarget(undefined)).toBe(false);
    expect(isEditableTarget({})).toBe(false);
  });
});

describe("isTriggerShortcut", () => {
  it("detects Alt+T with KeyT code", () => {
    expect(isTriggerShortcut({ altKey: true, code: "KeyT" })).toBe(true);
  });

  it("detects Alt+T with lowercase or uppercase t key", () => {
    expect(isTriggerShortcut({ altKey: true, key: "t" })).toBe(true);
    expect(isTriggerShortcut({ altKey: true, key: "T" })).toBe(true);
  });

  it("detects macOS Option+T special character dagger", () => {
    expect(isTriggerShortcut({ altKey: true, key: "†" })).toBe(true);
  });

  it("rejects without altKey modifier", () => {
    expect(isTriggerShortcut({ altKey: false, code: "KeyT" })).toBe(false);
    expect(isTriggerShortcut({ altKey: false, key: "t" })).toBe(false);
  });

  it("rejects different keys with altKey", () => {
    expect(isTriggerShortcut({ altKey: true, code: "KeyA", key: "a" })).toBe(false);
  });
});

describe("FocusTracker", () => {
  it("tracks and clears current active editable element", () => {
    const tracker = new FocusTracker();
    expect(tracker.getCurrent()).toBeNull();

    const mockTextarea = { tagName: "TEXTAREA" };
    tracker.onFocus(mockTextarea);
    expect(tracker.getCurrent()).toBe(mockTextarea as any);

    const mockDiv = { tagName: "DIV" };
    tracker.onFocus(mockDiv);
    // Non-editable element focus should not override active target
    expect(tracker.getCurrent()).toBe(mockTextarea as any);

    tracker.onBlur(mockTextarea);
    expect(tracker.getCurrent()).toBeNull();
  });
});
