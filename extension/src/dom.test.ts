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
});
