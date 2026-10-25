import { insertTextIntoTarget } from "./dom";

export enum OutputMode {
  HanziOnly = 0,
  HanziWithNumber = 1,
  HanziWithMarks = 2,
}

export interface ComposerBridge {
  init(): Promise<void> | void;
  preedit(): string;
  candidates(): string[];
  selected(): number;
  hints(): string[];
  pushChar(char: string): Promise<boolean> | boolean;
  backspace(): Promise<boolean> | boolean;
  cancel(): Promise<void> | void;
  moveSelection(delta: number): Promise<void> | void;
  commitIndex(index: number): Promise<string | null> | string | null;
  setOutputMode(mode: OutputMode): Promise<void> | void;
}

export class BackgroundBridge implements ComposerBridge {
  private state = {
    preedit: "",
    candidates: [] as string[],
    selected: 0,
    hints: [] as string[],
  };

  public async init(): Promise<void> {
    if (typeof chrome !== "undefined" && chrome?.runtime?.sendMessage) {
      const res = await chrome.runtime.sendMessage({ type: "INIT" });
      if (res?.ok) {
        this.updateState(res);
      }
    }
  }

  public preedit(): string {
    return this.state.preedit;
  }

  public candidates(): string[] {
    return this.state.candidates;
  }

  public selected(): number {
    return this.state.selected;
  }

  public hints(): string[] {
    return this.state.hints;
  }

  private updateState(res: any): void {
    if (res?.preedit !== undefined) this.state.preedit = res.preedit;
    if (res?.candidates !== undefined) this.state.candidates = res.candidates;
    if (res?.selected !== undefined) this.state.selected = res.selected;
    if (res?.hints !== undefined) this.state.hints = res.hints;
  }

  public async pushChar(char: string): Promise<boolean> {
    if (typeof chrome !== "undefined" && chrome?.runtime?.sendMessage) {
      const res = await chrome.runtime.sendMessage({ type: "PUSH_CHAR", char });
      if (res?.ok) {
        this.updateState(res);
        return Boolean(res.handled);
      }
    }
    return false;
  }

  public async backspace(): Promise<boolean> {
    if (typeof chrome !== "undefined" && chrome?.runtime?.sendMessage) {
      const res = await chrome.runtime.sendMessage({ type: "BACKSPACE" });
      if (res?.ok) {
        this.updateState(res);
        return Boolean(res.handled);
      }
    }
    return false;
  }

  public async cancel(): Promise<void> {
    this.state.preedit = "";
    this.state.candidates = [];
    this.state.selected = 0;
    this.state.hints = [];
    if (typeof chrome !== "undefined" && chrome?.runtime?.sendMessage) {
      const res = await chrome.runtime.sendMessage({ type: "CANCEL" });
      if (res?.ok) {
        this.updateState(res);
      }
    }
  }

  public async moveSelection(delta: number): Promise<void> {
    if (typeof chrome !== "undefined" && chrome?.runtime?.sendMessage) {
      const res = await chrome.runtime.sendMessage({ type: "MOVE_SELECTION", delta });
      if (res?.ok) {
        this.updateState(res);
      }
    }
  }

  public async commitIndex(index: number): Promise<string | null> {
    if (typeof chrome !== "undefined" && chrome?.runtime?.sendMessage) {
      const res = await chrome.runtime.sendMessage({ type: "COMMIT_INDEX", index });
      if (res?.ok) {
        this.updateState(res);
        return res.text ?? null;
      }
    }
    return null;
  }

  public async setOutputMode(mode: OutputMode): Promise<void> {
    if (typeof chrome !== "undefined" && chrome?.runtime?.sendMessage) {
      await chrome.runtime.sendMessage({ type: "SET_OUTPUT_MODE", mode });
    }
  }
}

class SyncComposerAdapter implements ComposerBridge {
  constructor(private composer: any) {}

  public init(): void {}

  public preedit(): string {
    return this.composer.preedit();
  }

  public candidates(): string[] {
    return this.composer.candidates();
  }

  public selected(): number {
    return this.composer.selected();
  }

  public hints(): string[] {
    return typeof this.composer.hints === "function" ? this.composer.hints() : [];
  }

  public pushChar(char: string): boolean {
    return Boolean(this.composer.push_char(char));
  }

  public backspace(): boolean {
    return Boolean(this.composer.backspace());
  }

  public cancel(): void {
    this.composer.cancel();
  }

  public moveSelection(delta: number): void {
    this.composer.move_selection(delta);
  }

  public commitIndex(index: number): string | null {
    return this.composer.commit_index(index) ?? null;
  }

  public setOutputMode(mode: OutputMode): void {
    if (typeof this.composer.set_output_mode === "function") {
      this.composer.set_output_mode(mode);
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
  private outputMode: OutputMode = OutputMode.HanziOnly;
  private bridge: ComposerBridge;
  private currentTarget: HTMLElement | null = null;
  private visible = false;

  constructor(bridgeOrComposer?: ComposerBridge | any) {
    if (bridgeOrComposer) {
      if (typeof bridgeOrComposer.pushChar === "function") {
        this.bridge = bridgeOrComposer;
      } else {
        this.bridge = new SyncComposerAdapter(bridgeOrComposer);
      }
    } else {
      this.bridge = new BackgroundBridge();
    }
  }

  public async init(): Promise<void> {
    const initRes = this.bridge.init();
    if (initRes instanceof Promise) {
      await initRes;
    }
    if (typeof chrome !== "undefined" && chrome?.storage?.sync) {
      try {
        const stored = await chrome.storage.sync.get("outputMode");
        if (typeof stored?.outputMode === "number") {
          this.outputMode = stored.outputMode as OutputMode;
          if (this.modeBadgeElem) {
            this.modeBadgeElem.textContent = this.getModeBadgeLabel();
          }
        }
      } catch (err) {
        console.error("[Taieagi Palette] Failed to load outputMode from storage:", err);
      }
    }
    this.createUI();
    this.setupStorageSync();
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

  public cycleOutputMode(): void {
    const nextMode =
      this.outputMode === OutputMode.HanziOnly
        ? OutputMode.HanziWithMarks
        : this.outputMode === OutputMode.HanziWithMarks
        ? OutputMode.HanziWithNumber
        : OutputMode.HanziOnly;
    this.setOutputMode(nextMode);
  }

  public setOutputMode(mode: OutputMode): void {
    this.outputMode = mode;
    this.bridge.setOutputMode(mode);
    if (this.modeBadgeElem) {
      this.modeBadgeElem.textContent = this.getModeBadgeLabel();
    }
    if (typeof chrome !== "undefined" && chrome?.storage?.sync) {
      chrome.storage.sync.set({ outputMode: mode }).catch((err) => {
        console.error("[Taieagi Palette] Failed to save outputMode to storage:", err);
      });
    }
  }

  private setupStorageSync(): void {
    if (typeof chrome !== "undefined" && chrome?.storage?.onChanged) {
      chrome.storage.onChanged.addListener((changes, areaName) => {
        if (areaName === "sync" && changes.outputMode !== undefined) {
          const newMode = changes.outputMode.newValue;
          if (typeof newMode === "number") {
            this.outputMode = newMode as OutputMode;
            if (this.modeBadgeElem) {
              this.modeBadgeElem.textContent = this.getModeBadgeLabel();
            }
          }
        }
      });
    }
  }

  public getOutputMode(): OutputMode {
    return this.outputMode;
  }

  private getModeBadgeLabel(): string {
    switch (this.outputMode) {
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
