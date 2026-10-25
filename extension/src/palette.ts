import type { ImeState } from "./background";
import { insertTextIntoTarget } from "./dom";

export enum OutputMode {
  HanziOnly = 0,
  HanziWithNumber = 1,
  HanziWithMarks = 2,
}

export interface ComposerBridge {
  init(onStateChange?: () => void): Promise<void> | void;
  preedit(): string;
  candidates(): string[];
  selected(): number;
  hints(): string[];
  outputMode(): OutputMode;
  pushChar(char: string): Promise<boolean> | boolean;
  backspace(): Promise<boolean> | boolean;
  cancel(): Promise<void> | void;
  moveSelection(delta: number): Promise<void> | void;
  commitIndex(index: number): Promise<string | null> | string | null;
  setOutputMode(mode: OutputMode): Promise<void> | void;
}

export class BackgroundBridge implements ComposerBridge {
  private session: ImeState = {
    preedit: "",
    candidates: [],
    selected: 0,
    hints: [],
  };
  private currentMode: OutputMode = OutputMode.HanziOnly;

  public async init(onStateChange?: () => void): Promise<void> {
    if (typeof chrome !== "undefined" && chrome?.runtime?.sendMessage) {
      const res = await chrome.runtime.sendMessage({ type: "INIT" });
      if (res?.ok) {
        this.updateSession(res);
        if (typeof res.outputMode === "number") {
          this.currentMode = res.outputMode as OutputMode;
        }
      }
    }
    if (typeof chrome !== "undefined" && chrome?.storage?.onChanged) {
      chrome.storage.onChanged.addListener((changes, areaName) => {
        if (areaName === "sync" && changes.outputMode !== undefined) {
          const newMode = changes.outputMode.newValue;
          if (typeof newMode === "number") {
            this.currentMode = newMode as OutputMode;
            onStateChange?.();
          }
        }
      });
    }
  }

  public preedit(): string {
    return this.session.preedit;
  }

  public candidates(): string[] {
    return this.session.candidates;
  }

  public selected(): number {
    return this.session.selected;
  }

  public hints(): string[] {
    return this.session.hints;
  }

  public outputMode(): OutputMode {
    return this.currentMode;
  }

  private updateSession(res: Partial<ImeState>): void {
    if (res.preedit !== undefined) this.session.preedit = res.preedit;
    if (res.candidates !== undefined) this.session.candidates = res.candidates;
    if (res.selected !== undefined) this.session.selected = res.selected;
    if (res.hints !== undefined) this.session.hints = res.hints;
  }

  public async pushChar(char: string): Promise<boolean> {
    if (typeof chrome !== "undefined" && chrome?.runtime?.sendMessage) {
      const res = await chrome.runtime.sendMessage({ type: "PUSH_CHAR", char });
      if (res?.ok) {
        this.updateSession(res);
        return Boolean(res.handled);
      }
    }
    return false;
  }

  public async backspace(): Promise<boolean> {
    if (typeof chrome !== "undefined" && chrome?.runtime?.sendMessage) {
      const res = await chrome.runtime.sendMessage({ type: "BACKSPACE" });
      if (res?.ok) {
        this.updateSession(res);
        return Boolean(res.handled);
      }
    }
    return false;
  }

  public async cancel(): Promise<void> {
    this.session = {
      preedit: "",
      candidates: [],
      selected: 0,
      hints: [],
    };
    if (typeof chrome !== "undefined" && chrome?.runtime?.sendMessage) {
      const res = await chrome.runtime.sendMessage({ type: "CANCEL" });
      if (res?.ok) {
        this.updateSession(res);
      }
    }
  }

  public async moveSelection(delta: number): Promise<void> {
    if (typeof chrome !== "undefined" && chrome?.runtime?.sendMessage) {
      const res = await chrome.runtime.sendMessage({ type: "MOVE_SELECTION", delta });
      if (res?.ok) {
        this.updateSession(res);
      }
    }
  }

  public async commitIndex(index: number): Promise<string | null> {
    if (typeof chrome !== "undefined" && chrome?.runtime?.sendMessage) {
      const res = await chrome.runtime.sendMessage({ type: "COMMIT_INDEX", index });
      if (res?.ok) {
        this.updateSession(res);
        return res.text ?? null;
      }
    }
    return null;
  }

  public async setOutputMode(mode: OutputMode): Promise<void> {
    this.currentMode = mode;
    if (typeof chrome !== "undefined" && chrome?.runtime?.sendMessage) {
      const res = await chrome.runtime.sendMessage({ type: "SET_OUTPUT_MODE", mode });
      if (res?.ok && typeof res.outputMode === "number") {
        this.currentMode = res.outputMode as OutputMode;
      }
    }
  }
}

export const CANDIDATE_PAGE_SIZE = 9;

export class Palette {
  private container: HTMLDivElement | null = null;
  private preeditElem: HTMLSpanElement | null = null;
  private pageIndicatorElem: HTMLSpanElement | null = null;
  private candidatesElem: HTMLDivElement | null = null;
  private hintsElem: HTMLDivElement | null = null;
  private modeBadgeElem: HTMLButtonElement | null = null;
  private bridge: ComposerBridge;
  private currentTarget: HTMLElement | null = null;
  private visible = false;

  constructor(bridge: ComposerBridge = new BackgroundBridge()) {
    this.bridge = bridge;
  }

  public async init(): Promise<void> {
    const initRes = this.bridge.init(() => {
      this.updateModeBadge();
    });
    if (initRes instanceof Promise) {
      await initRes;
    }
    this.createUI();
    this.updateModeBadge();
  }

  public isOpen(): boolean {
    return this.visible;
  }

  public getTarget(): HTMLElement | null {
    return this.currentTarget;
  }

  public toggle(target: HTMLElement): void {
    if (this.visible && this.currentTarget === target) {
      this.close();
    } else {
      this.open(target);
    }
  }

  public open(target: HTMLElement): void {
    this.currentTarget = target;
    const cancelRes = this.bridge.cancel();
    if (cancelRes instanceof Promise) {
      cancelRes.catch(() => {});
    }
    this.visible = true;

    if (!this.container) {
      this.createUI();
    }

    this.positionPalette(target);
    if (this.container) {
      this.container.style.display = "block";
    }
    this.render();
  }

  public close(): void {
    this.visible = false;
    if (this.container) {
      this.container.style.display = "none";
    }
    const cancelRes = this.bridge.cancel();
    if (cancelRes instanceof Promise) {
      cancelRes.catch(() => {});
    }
    this.currentTarget = null;
  }

  private cycleOutputMode(): void {
    const current = this.bridge.outputMode();
    const nextMode =
      current === OutputMode.HanziOnly
        ? OutputMode.HanziWithMarks
        : current === OutputMode.HanziWithMarks
        ? OutputMode.HanziWithNumber
        : OutputMode.HanziOnly;
    const res = this.bridge.setOutputMode(nextMode);
    this.updateModeBadge();
    if (res instanceof Promise) {
      res.then(() => {
        this.updateModeBadge();
      });
    }
  }

  private updateModeBadge(): void {
    if (this.modeBadgeElem) {
      this.modeBadgeElem.textContent = this.getModeBadgeLabel();
    }
  }

  private getModeBadgeLabel(): string {
    switch (this.bridge.outputMode()) {
      case OutputMode.HanziWithMarks:
        return "[漢+調]";
      case OutputMode.HanziWithNumber:
        return "[漢+數]";
      case OutputMode.HanziOnly:
      default:
        return "[純漢]";
    }
  }

  private createUI(): void {
    if (this.container || typeof document === "undefined") {
      return;
    }

    const container = document.createElement("div");
    container.id = "taieagi-palette";
    container.className = "taieagi-palette";
    container.style.display = "none";

    // Prevent clicking inside palette from stealing focus from target element
    container.addEventListener("mousedown", (e) => {
      e.preventDefault();
      e.stopPropagation();
    });

    const preeditBar = document.createElement("div");
    preeditBar.className = "taieagi-preedit-bar";

    const preeditText = document.createElement("span");
    preeditText.className = "taieagi-preedit-text";
    preeditBar.appendChild(preeditText);
    this.preeditElem = preeditText;

    const pageIndicator = document.createElement("span");
    pageIndicator.className = "taieagi-page-indicator";
    preeditBar.appendChild(pageIndicator);
    this.pageIndicatorElem = pageIndicator;

    const modeBadge = document.createElement("button");
    modeBadge.type = "button";
    modeBadge.className = "taieagi-mode-badge";
    modeBadge.title = "切換輸出模式：純漢 - 漢+調 - 漢+數";
    modeBadge.textContent = this.getModeBadgeLabel();
    modeBadge.addEventListener("click", (e) => {
      e.preventDefault();
      e.stopPropagation();
      this.cycleOutputMode();
    });
    preeditBar.appendChild(modeBadge);
    this.modeBadgeElem = modeBadge;

    const candidatesBar = document.createElement("div");
    candidatesBar.className = "taieagi-candidates-bar";
    this.candidatesElem = candidatesBar;

    const hintsBar = document.createElement("div");
    hintsBar.className = "taieagi-hints-panel";
    hintsBar.style.display = "none";
    this.hintsElem = hintsBar;

    container.appendChild(preeditBar);
    container.appendChild(candidatesBar);
    container.appendChild(hintsBar);

    document.body.appendChild(container);
    this.container = container;
  }

  private positionPalette(target: HTMLElement): void {
    if (!this.container) {
      return;
    }

    const rect = target.getBoundingClientRect();
    let top = rect.bottom + 6;
    let left = rect.left;

    const viewportHeight = typeof window !== "undefined" ? window.innerHeight : 800;
    const estimatedHeight = 72;

    // Flip above target if overflowing the bottom of the viewport
    if (rect.bottom + estimatedHeight > viewportHeight && rect.top > estimatedHeight) {
      top = rect.top - estimatedHeight - 6;
    }

    this.container.style.top = `${Math.max(0, top)}px`;
    this.container.style.left = `${Math.max(0, left)}px`;
  }

  public render(): void {
    if (!this.preeditElem || !this.candidatesElem) {
      return;
    }

    const preedit = this.bridge.preedit();
    if (preedit.length === 0) {
      this.preeditElem.textContent = "臺羅拼音 (如 tsui2)";
      this.preeditElem.className = "taieagi-preedit-text placeholder";
    } else {
      this.preeditElem.textContent = preedit;
      this.preeditElem.className = "taieagi-preedit-text";
    }

    const candidates = this.bridge.candidates();
    const selectedIdx = this.bridge.selected();

    this.candidatesElem.replaceChildren();
    this.renderHints();

    if (candidates.length === 0) {
      if (this.pageIndicatorElem) {
        this.pageIndicatorElem.textContent = "";
      }
      if (preedit.length > 0) {
        const emptyMsg = document.createElement("span");
        emptyMsg.className = "taieagi-empty-hint";
        emptyMsg.textContent = "(無對應漢字)";
        this.candidatesElem.appendChild(emptyMsg);
      }
      return;
    }

    const pageIndex = Math.floor(selectedIdx / CANDIDATE_PAGE_SIZE);
    const totalPages = Math.ceil(candidates.length / CANDIDATE_PAGE_SIZE);
    const pageStart = pageIndex * CANDIDATE_PAGE_SIZE;
    const pageCandidates = candidates.slice(pageStart, pageStart + CANDIDATE_PAGE_SIZE);

    if (this.pageIndicatorElem) {
      this.pageIndicatorElem.textContent = totalPages > 1 ? `${pageIndex + 1}/${totalPages}` : "";
    }

    pageCandidates.forEach((cand, localIdx) => {
      const globalIdx = pageStart + localIdx;
      const item = document.createElement("div");
      item.className = "taieagi-candidate-item" + (globalIdx === selectedIdx ? " active" : "");

      const num = document.createElement("span");
      num.className = "taieagi-candidate-num";
      num.textContent = `${localIdx + 1}.`;

      const text = document.createElement("span");
      text.className = "taieagi-candidate-word";
      text.textContent = cand;

      item.appendChild(num);
      item.appendChild(text);

      item.addEventListener("mousedown", (e) => {
        e.preventDefault();
        e.stopPropagation();
      });

      item.addEventListener("click", (e) => {
        e.preventDefault();
        e.stopPropagation();
        this.commitIndex(globalIdx);
      });

      this.candidatesElem?.appendChild(item);
    });
  }

  private renderHints(): void {
    if (!this.hintsElem) {
      return;
    }

    const hints = this.bridge.hints();
    this.hintsElem.replaceChildren();

    if (hints.length > 0) {
      this.hintsElem.style.display = "flex";
      const title = document.createElement("span");
      title.className = "taieagi-hints-title";
      title.textContent = "你想輸入";
      this.hintsElem.appendChild(title);

      hints.forEach((hint) => {
        const item = document.createElement("span");
        item.className = "taieagi-hint-item";
        item.textContent = hint;
        this.hintsElem?.appendChild(item);
      });

      const suffix = document.createElement("span");
      suffix.className = "taieagi-hints-suffix";
      suffix.textContent = "嗎？";
      this.hintsElem.appendChild(suffix);
    } else {
      this.hintsElem.style.display = "none";
    }
  }

  public commitIndex(index: number): void {
    const res = this.bridge.commitIndex(index);
    if (res instanceof Promise) {
      res.then((text) => {
        if (text) {
          this.commitText(text);
        }
      });
    } else if (res) {
      this.commitText(res);
    }
  }

  private commitText(text: string): void {
    if (this.currentTarget) {
      insertTextIntoTarget(this.currentTarget, text);
    }
    this.close();
  }

  private moveAndRender(delta: number): void {
    const res = this.bridge.moveSelection(delta);
    if (res instanceof Promise) {
      res.then(() => {
        this.render();
      });
    } else {
      this.render();
    }
  }

  public handleKeyDown(e: KeyboardEvent): boolean {
    if (!this.visible) {
      return false;
    }

    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      if (this.bridge.preedit().length > 0) {
        const cancelRes = this.bridge.cancel();
        if (cancelRes instanceof Promise) {
          cancelRes.then(() => {
            this.render();
          });
        } else {
          this.render();
        }
      } else {
        this.close();
      }
      return true;
    }

    if (e.key === "ArrowRight" || (e.key === "Tab" && !e.shiftKey)) {
      e.preventDefault();
      e.stopPropagation();
      this.moveAndRender(1);
      return true;
    }

    if (e.key === "ArrowLeft" || (e.key === "Tab" && e.shiftKey)) {
      e.preventDefault();
      e.stopPropagation();
      this.moveAndRender(-1);
      return true;
    }

    if (e.key === "ArrowDown" || e.key === "PageDown") {
      e.preventDefault();
      e.stopPropagation();
      this.moveAndRender(CANDIDATE_PAGE_SIZE);
      return true;
    }

    if (e.key === "ArrowUp" || e.key === "PageUp") {
      e.preventDefault();
      e.stopPropagation();
      this.moveAndRender(-CANDIDATE_PAGE_SIZE);
      return true;
    }

    if (e.key === " " || e.key === "Enter") {
      const candidates = this.bridge.candidates();
      if (candidates.length > 0) {
        e.preventDefault();
        e.stopPropagation();
        this.commitIndex(this.bridge.selected());
        return true;
      }
      if (this.bridge.preedit().length > 0) {
        e.preventDefault();
        e.stopPropagation();
        return true;
      }
      return false;
    }

    if (e.key === "Backspace") {
      e.preventDefault();
      e.stopPropagation();
      const prevPreedit = this.bridge.preedit();
      if (prevPreedit.length === 0) {
        return true;
      }
      const res = this.bridge.backspace();
      if (res instanceof Promise) {
        res.then(() => {
          this.render();
        });
      } else {
        this.render();
      }
      return true;
    }

    // Pass through combination shortcuts like Ctrl+A, Cmd+C, etc.
    if (e.ctrlKey || e.metaKey || e.altKey) {
      return false;
    }

    if (e.key.length === 1) {
      const char = e.key.toLowerCase();
      // Allow letters and tone digits
      if ((char >= "a" && char <= "z") || (char >= "1" && char <= "9") || char === "-") {
        e.preventDefault();
        e.stopPropagation();
        const res = this.bridge.pushChar(char);
        if (res instanceof Promise) {
          res.then(() => {
            this.render();
          });
        } else {
          this.render();
        }
        return true;
      }
    }

    return false;
  }
}
