import { describe, expect, it } from "bun:test";
import { insertTextIntoTarget } from "./dom";

describe("insertTextIntoTarget", () => {
  it("inserts text into textarea using fallback when execCommand is unavailable", () => {
    let dispatchedEvents: string[] = [];
    const mockTextarea = {
      tagName: "TEXTAREA",
      value: "hello world",
      selectionStart: 5,
      selectionEnd: 5,
      focus: () => {},
      dispatchEvent: (event: any) => {
        dispatchedEvents.push(event.type);
        return true;
      },
    } as any;

    const result = insertTextIntoTarget(mockTextarea, " there");
    expect(result).toBe(true);
    expect(mockTextarea.value).toBe("hello there world");
    expect(mockTextarea.selectionStart).toBe(11);
    expect(mockTextarea.selectionEnd).toBe(11);
    expect(dispatchedEvents).toContain("input");
    expect(dispatchedEvents).toContain("change");
  });

  it("replaces selected range in input", () => {
    const mockInput = {
      tagName: "INPUT",
      value: "test replace",
      selectionStart: 5,
      selectionEnd: 12,
      focus: () => {},
      dispatchEvent: () => true,
    } as any;

    const result = insertTextIntoTarget(mockInput, "success");
    expect(result).toBe(true);
    expect(mockInput.value).toBe("test success");
    expect(mockInput.selectionStart).toBe(12);
    expect(mockInput.selectionEnd).toBe(12);
  });

  it("inserts text into contenteditable via synthetic paste when intercepted by rich-text framework", () => {
    let pastedText = "";
    let isPrevented = false;

    const originalClipboardEvent = (globalThis as any).ClipboardEvent;
    const originalDataTransfer = (globalThis as any).DataTransfer;

    class MockDataTransfer {
      private data: Record<string, string> = {};
      public setData(format: string, data: string) {
        this.data[format] = data;
      }
      public getData(format: string) {
        return this.data[format] || "";
      }
    }

    class MockClipboardEvent {
      public type: string;
      public clipboardData: MockDataTransfer;
      public defaultPrevented = false;

      constructor(type: string, init?: any) {
        this.type = type;
        this.clipboardData = init?.clipboardData;
      }

      public preventDefault() {
        this.defaultPrevented = true;
      }
    }

    (globalThis as any).ClipboardEvent = MockClipboardEvent;
    (globalThis as any).DataTransfer = MockDataTransfer;

    try {
      const mockContentEditable = {
        isContentEditable: true,
        focus: () => {},
        dispatchEvent: (event: any) => {
          if (event.type === "paste") {
            pastedText = event.clipboardData?.getData("text/plain");
            event.preventDefault(); // Simulate framework intercepting paste
            isPrevented = event.defaultPrevented;
            return false;
          }
          return true;
        },
      } as any;

      const result = insertTextIntoTarget(mockContentEditable, "水");
      expect(result).toBe(true);
      expect(pastedText).toBe("水");
      expect(isPrevented).toBe(true);
    } finally {
      (globalThis as any).ClipboardEvent = originalClipboardEvent;
      (globalThis as any).DataTransfer = originalDataTransfer;
    }
  });
});
