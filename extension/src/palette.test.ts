import { describe, expect, it } from "bun:test";
import { CANDIDATE_PAGE_SIZE, OutputMode, Palette } from "./palette";

describe("Palette", () => {
  function createMockComposer() {
    let preeditStr = "";
    let candidatesList: string[] = [];
    let selectedIdx = 0;
    let hintsList: string[] = [];

    return {
      preedit: () => preeditStr,
      candidates: () => candidatesList,
      selected: () => selectedIdx,
      hints: () => hintsList,
      cancel: () => {
        preeditStr = "";
        candidatesList = [];
        selectedIdx = 0;
        hintsList = [];
        return true;
      },
      push_char: (c: string) => {
        preeditStr += c;
        if (preeditStr === "tsui") {
          candidatesList = ["水", "媠", "嘴"];
        } else if (preeditStr === "a") {
          candidatesList = Array.from({ length: 12 }, (_, i) => `字${i + 1}`);
        } else if (preeditStr === "jire") {
          hintsList = ["這个 (tsit-ê)"];
        }
        return true;
      },
      backspace: () => {
        preeditStr = preeditStr.slice(0, -1);
        return true;
      },
      move_selection: (delta: number) => {
        selectedIdx = Math.max(0, Math.min(candidatesList.length - 1, selectedIdx + delta));
        return true;
      },
      commit_selected: () => {
        if (candidatesList.length > 0) {
          const text = candidatesList[selectedIdx];
          preeditStr = "";
          candidatesList = [];
          return text;
        }
        return undefined;
      },
      output_mode: () => 0,
      set_output_mode: () => {},
      free: () => {},
      [Symbol.dispose]: () => {},
    } as any;
  }

  it("opens, closes, and toggles relative to target element", () => {
    const mockComposer = createMockComposer();
    const palette = new Palette(mockComposer);
    const mockTarget = {
      tagName: "TEXTAREA",
      getBoundingClientRect: () => ({ top: 100, bottom: 130, left: 50, right: 250, width: 200, height: 30 }),
      focus: () => {},
    } as any;

    expect(palette.isOpen()).toBe(false);

    palette.open(mockTarget);
    expect(palette.isOpen()).toBe(true);
    expect(palette.getTarget()).toBe(mockTarget);

    palette.toggle(mockTarget);
    expect(palette.isOpen()).toBe(false);
    expect(palette.getTarget()).toBeNull();
  });

  it("handles keyboard events: typing letters, navigating candidates, and committing", () => {
    const mockComposer = createMockComposer();
    const palette = new Palette(mockComposer);
    const mockTarget = {
      tagName: "TEXTAREA",
      value: "",
      selectionStart: 0,
      selectionEnd: 0,
      getBoundingClientRect: () => ({ top: 100, bottom: 130, left: 50, right: 250, width: 200, height: 30 }),
      focus: () => {},
      dispatchEvent: () => true,
    } as any;

    palette.open(mockTarget);

    // Type 't', 's', 'u', 'i'
    for (const char of "tsui") {
      const handled = palette.handleKeyDown({
        key: char,
        preventDefault: () => {},
        stopPropagation: () => {},
      } as any);
      expect(handled).toBe(true);
    }

    expect(mockComposer.preedit()).toBe("tsui");
    expect(mockComposer.candidates()).toEqual(["水", "媠", "嘴"]);

    // Navigate to candidate 2 ("媠")
    const navHandled = palette.handleKeyDown({
      key: "ArrowRight",
      preventDefault: () => {},
      stopPropagation: () => {},
    } as any);
    expect(navHandled).toBe(true);
    expect(mockComposer.selected()).toBe(1);

    // Commit with Space
    const commitHandled = palette.handleKeyDown({
      key: " ",
      preventDefault: () => {},
      stopPropagation: () => {},
    } as any);
    expect(commitHandled).toBe(true);
    expect(mockTarget.value).toBe("媠");
    expect(palette.isOpen()).toBe(false);
  });

  it("navigates across pages by CANDIDATE_PAGE_SIZE using ArrowDown/ArrowUp and Shift+Tab", () => {
    const mockComposer = createMockComposer();
    const palette = new Palette(mockComposer);
    const mockTarget = {
      tagName: "TEXTAREA",
      value: "",
      selectionStart: 0,
      selectionEnd: 0,
      getBoundingClientRect: () => ({ top: 100, bottom: 130, left: 50, right: 250, width: 200, height: 30 }),
      focus: () => {},
      dispatchEvent: () => true,
    } as any;

    palette.open(mockTarget);

    palette.handleKeyDown({
      key: "a",
      preventDefault: () => {},
      stopPropagation: () => {},
    } as any);

    expect(CANDIDATE_PAGE_SIZE).toBe(9);
    expect(mockComposer.selected()).toBe(0);

    // ArrowDown jumps to second page (index 9 -> "字10")
    palette.handleKeyDown({
      key: "ArrowDown",
      preventDefault: () => {},
      stopPropagation: () => {},
    } as any);
    expect(mockComposer.selected()).toBe(9);

    // Shift+Tab steps back by 1 to index 8 ("字9", end of first page)
    palette.handleKeyDown({
      key: "Tab",
      shiftKey: true,
      preventDefault: () => {},
      stopPropagation: () => {},
    } as any);
    expect(mockComposer.selected()).toBe(8);

    // PageDown clamps to last item (index 11 -> "字12")
    palette.handleKeyDown({
      key: "PageDown",
      preventDefault: () => {},
      stopPropagation: () => {},
    } as any);
    expect(mockComposer.selected()).toBe(11);

    // ArrowUp jumps back by 9 to index 2 ("字3")
    palette.handleKeyDown({
      key: "ArrowUp",
      preventDefault: () => {},
      stopPropagation: () => {},
    } as any);
    expect(mockComposer.selected()).toBe(2);
  });

  it("closes on Escape key when buffer is empty", () => {
    const mockComposer = createMockComposer();
    const palette = new Palette(mockComposer);
    const mockTarget = {
      getBoundingClientRect: () => ({ top: 0, bottom: 20, left: 0, right: 100, width: 100, height: 20 }),
    } as any;

    palette.open(mockTarget);
    expect(palette.isOpen()).toBe(true);

    const handled = palette.handleKeyDown({
      key: "Escape",
      preventDefault: () => {},
      stopPropagation: () => {},
    } as any);

    expect(handled).toBe(true);
    expect(palette.isOpen()).toBe(false);
  });

  it("clears buffer on Escape when buffer is not empty, and closes on subsequent Escape", () => {
    const mockComposer = createMockComposer();
    const palette = new Palette(mockComposer);
    const mockTarget = {
      getBoundingClientRect: () => ({ top: 0, bottom: 20, left: 0, right: 100, width: 100, height: 20 }),
    } as any;

    palette.open(mockTarget);
    expect(palette.isOpen()).toBe(true);

    palette.handleKeyDown({
      key: "a",
      preventDefault: () => {},
      stopPropagation: () => {},
    } as any);
    expect(mockComposer.preedit()).toBe("a");

    // First Escape: buffer is non-empty, so clear buffer and stay open
    let handled = palette.handleKeyDown({
      key: "Escape",
      preventDefault: () => {},
      stopPropagation: () => {},
    } as any);
    expect(handled).toBe(true);
    expect(mockComposer.preedit()).toBe("");
    expect(palette.isOpen()).toBe(true);

    // Second Escape: buffer is now empty, so close palette
    handled = palette.handleKeyDown({
      key: "Escape",
      preventDefault: () => {},
      stopPropagation: () => {},
    } as any);
    expect(handled).toBe(true);
    expect(palette.isOpen()).toBe(false);
  });

  it("keeps palette open when Backspace is pressed on empty buffer", () => {
    const mockComposer = createMockComposer();
    const palette = new Palette(mockComposer);
    const mockTarget = {
      getBoundingClientRect: () => ({ top: 0, bottom: 20, left: 0, right: 100, width: 100, height: 20 }),
    } as any;

    palette.open(mockTarget);
    expect(palette.isOpen()).toBe(true);

    let prevented = false;
    let stopped = false;
    const handled = palette.handleKeyDown({
      key: "Backspace",
      preventDefault: () => {
        prevented = true;
      },
      stopPropagation: () => {
        stopped = true;
      },
    } as any);

    expect(handled).toBe(true);
    expect(prevented).toBe(true);
    expect(stopped).toBe(true);
    expect(palette.isOpen()).toBe(true);
  });

  it("renders hints panel when hints are present and swallows Space/Enter when no candidates", () => {
    const createdElements: any[] = [];
    (globalThis as any).document = {
      createElement: (tag: string) => {
        const el: any = {
          tagName: tag.toUpperCase(),
          className: "",
          id: "",
          style: {},
          children: [] as any[],
          textContent: "",
          appendChild: (child: any) => {
            el.children.push(child);
            return child;
          },
          replaceChildren: (...newChildren: any[]) => {
            el.children = [...newChildren];
          },
          addEventListener: () => {},
        };
        createdElements.push(el);
        return el;
      },
      getElementById: (id: string) => createdElements.find((e) => e.id === id) ?? null,
      body: {
        appendChild: () => {},
      },
    };

    try {
      const mockComposer = createMockComposer();
      const palette = new Palette(mockComposer);
      const mockTarget = {
        getBoundingClientRect: () => ({ top: 0, bottom: 20, left: 0, right: 100, width: 100, height: 20 }),
      } as any;

      palette.open(mockTarget);

      for (const char of "jire") {
        palette.handleKeyDown({
          key: char,
          preventDefault: () => {},
          stopPropagation: () => {},
        } as any);
      }

      const container = (globalThis as any).document.getElementById("taieagi-palette");
      expect(container).not.toBeNull();

      const hintsPanel = container?.children.find((c: any) => c.className === "taieagi-hints-panel");
      expect(hintsPanel).toBeDefined();
      expect(hintsPanel.style.display).toBe("flex");
      const title = hintsPanel.children.find((c: any) => c.className === "taieagi-hints-title");
      expect(title?.textContent).toBe("你想輸入");
      const hintItem = hintsPanel.children.find((c: any) => c.className === "taieagi-hint-item");
      expect(hintItem?.textContent).toBe("這个 (tsit-ê)");
      const suffix = hintsPanel.children.find((c: any) => c.className === "taieagi-hints-suffix");
      expect(suffix?.textContent).toBe("嗎？");

      let spacePrevented = false;
      const handledSpace = palette.handleKeyDown({
        key: " ",
        preventDefault: () => {
          spacePrevented = true;
        },
        stopPropagation: () => {},
      } as any);
      expect(handledSpace).toBe(true);
      expect(spacePrevented).toBe(true);
      expect(palette.isOpen()).toBe(true);

      let enterPrevented = false;
      const handledEnter = palette.handleKeyDown({
        key: "Enter",
        preventDefault: () => {
          enterPrevented = true;
        },
        stopPropagation: () => {},
      } as any);
      expect(handledEnter).toBe(true);
      expect(enterPrevented).toBe(true);
      expect(palette.isOpen()).toBe(true);
    } finally {
      delete (globalThis as any).document;
    }
  });

  it("cycles output mode when mode badge is clicked", () => {
    const createdElements: any[] = [];
    (globalThis as any).document = {
      createElement: (tag: string) => {
        const el: any = {
          tagName: tag.toUpperCase(),
          className: "",
          id: "",
          style: {},
          children: [] as any[],
          textContent: "",
          appendChild: (child: any) => {
            el.children.push(child);
            return child;
          },
          replaceChildren: (...newChildren: any[]) => {
            el.children = [...newChildren];
          },
          addEventListener: (event: string, handler: any) => {
            if (event === "click") {
              el.click = () => handler({ preventDefault: () => {}, stopPropagation: () => {} });
            }
          },
        };
        createdElements.push(el);
        return el;
      },
      getElementById: (id: string) => createdElements.find((e) => e.id === id) ?? null,
      body: {
        appendChild: () => {},
      },
    };

    try {
      const mockComposer = createMockComposer();
      let setModeCalledWith: OutputMode | null = null;
      mockComposer.set_output_mode = (mode: number) => {
        setModeCalledWith = mode as OutputMode;
      };

      const palette = new Palette(mockComposer);
      const mockTarget = {
        getBoundingClientRect: () => ({ top: 0, bottom: 20, left: 0, right: 100, width: 100, height: 20 }),
      } as any;

      palette.open(mockTarget);

      const badge = createdElements.find((e) => e.className === "taieagi-mode-badge");
      expect(badge).toBeDefined();
      expect(badge.textContent).toBe("[純漢]");
      expect(palette.getOutputMode()).toBe(OutputMode.HanziOnly);

      // 1st click: HanziOnly -> HanziWithMarks
      badge.click();
      expect(palette.getOutputMode()).toBe(OutputMode.HanziWithMarks);
      expect(badge.textContent).toBe("[漢+調]");
      expect(setModeCalledWith).toBe(OutputMode.HanziWithMarks);

      // 2nd click: HanziWithMarks -> HanziWithNumber
      badge.click();
      expect(palette.getOutputMode()).toBe(OutputMode.HanziWithNumber);
      expect(badge.textContent).toBe("[漢+數]");
      expect(setModeCalledWith).toBe(OutputMode.HanziWithNumber);

      // 3rd click: HanziWithNumber -> HanziOnly
      badge.click();
      expect(palette.getOutputMode()).toBe(OutputMode.HanziOnly);
      expect(badge.textContent).toBe("[純漢]");
      expect(setModeCalledWith).toBe(OutputMode.HanziOnly);
    } finally {
      delete (globalThis as any).document;
    }
  });

  it("restores outputMode from chrome.storage.sync on init", async () => {
    const createdElements: any[] = [];
    (globalThis as any).document = {
      createElement: (tag: string) => {
        const el: any = {
          tagName: tag.toUpperCase(),
          className: "",
          id: "",
          style: {},
          children: [] as any[],
          textContent: "",
          appendChild: (child: any) => {
            el.children.push(child);
            return child;
          },
          replaceChildren: (...newChildren: any[]) => {
            el.children = [...newChildren];
          },
          addEventListener: () => {},
        };
        createdElements.push(el);
        return el;
      },
      getElementById: (id: string) => createdElements.find((e) => e.id === id) ?? null,
      body: { appendChild: () => {} },
    };

    (globalThis as any).chrome = {
      storage: {
        sync: {
          get: async (key: string) => {
            if (key === "outputMode") {
              return { outputMode: OutputMode.HanziWithMarks };
            }
            return {};
          },
          set: async () => {},
        },
        onChanged: {
          addListener: () => {},
        },
      },
    };

    try {
      const mockComposer = createMockComposer();
      const palette = new Palette(mockComposer);
      await palette.init();

      const mockTarget = {
        getBoundingClientRect: () => ({ top: 0, bottom: 20, left: 0, right: 100, width: 100, height: 20 }),
      } as any;

      palette.open(mockTarget);

      expect(palette.getOutputMode()).toBe(OutputMode.HanziWithMarks);
      const badge = createdElements.find((e) => e.className === "taieagi-mode-badge");
      expect(badge).toBeDefined();
      expect(badge.textContent).toBe("[漢+調]");
    } finally {
      delete (globalThis as any).document;
      delete (globalThis as any).chrome;
    }
  });

  it("updates outputMode and badge when chrome.storage.onChanged fires", async () => {
    const createdElements: any[] = [];
    (globalThis as any).document = {
      createElement: (tag: string) => {
        const el: any = {
          tagName: tag.toUpperCase(),
          className: "",
          id: "",
          style: {},
          children: [] as any[],
          textContent: "",
          appendChild: (child: any) => {
            el.children.push(child);
            return child;
          },
          replaceChildren: (...newChildren: any[]) => {
            el.children = [...newChildren];
          },
          addEventListener: () => {},
        };
        createdElements.push(el);
        return el;
      },
      getElementById: (id: string) => createdElements.find((e) => e.id === id) ?? null,
      body: { appendChild: () => {} },
    };

    let storageListener: ((changes: any, areaName: string) => void) | null = null;
    let savedStorageData: any = null;

    (globalThis as any).chrome = {
      storage: {
        sync: {
          get: async () => ({}),
          set: async (data: any) => {
            savedStorageData = data;
          },
        },
        onChanged: {
          addListener: (fn: any) => {
            storageListener = fn;
          },
        },
      },
    };

    try {
      const mockComposer = createMockComposer();
      const palette = new Palette(mockComposer);
      await palette.init();

      const mockTarget = {
        getBoundingClientRect: () => ({ top: 0, bottom: 20, left: 0, right: 100, width: 100, height: 20 }),
      } as any;

      palette.open(mockTarget);

      const badge = createdElements.find((e) => e.className === "taieagi-mode-badge");
      expect(badge.textContent).toBe("[純漢]");
      expect(palette.getOutputMode()).toBe(OutputMode.HanziOnly);

      // Verify saving to storage when setOutputMode is called
      palette.setOutputMode(OutputMode.HanziWithNumber);
      expect(savedStorageData).toEqual({ outputMode: OutputMode.HanziWithNumber });
      expect(badge.textContent).toBe("[漢+數]");

      // Verify onChanged updates mode and badge text
      expect(storageListener).not.toBeNull();
      storageListener!(
        { outputMode: { newValue: OutputMode.HanziWithMarks } },
        "sync"
      );
      expect(palette.getOutputMode()).toBe(OutputMode.HanziWithMarks);
      expect(badge.textContent).toBe("[漢+調]");
    } finally {
      delete (globalThis as any).document;
      delete (globalThis as any).chrome;
    }
  });
});
