import init, { WasmComposer } from "../pkg/taieagi.js";

export interface ImeState {
  preedit: string;
  candidates: string[];
  selected: number;
  hints: string[];
}

export type ComposerAction =
  | { type: "PUSH_CHAR"; char: string }
  | { type: "BACKSPACE" }
  | { type: "CANCEL" }
  | { type: "MOVE_SELECTION"; delta: number }
  | { type: "COMMIT_SELECTED" }
  | { type: "COMMIT_INDEX"; index: number };

export type ConfigAction =
  | { type: "SET_OUTPUT_MODE"; mode: number };

export type BackgroundMessage =
  | { type: "INIT" }
  | ComposerAction
  | ConfigAction;

let composer: WasmComposer | null = null;
let initPromise: Promise<void> | null = null;

export async function ensureInitialized(): Promise<WasmComposer> {
  if (composer) {
    return composer;
  }

  if (!initPromise) {
    initPromise = (async () => {
      if (typeof chrome !== "undefined" && chrome?.runtime?.getURL) {
        const wasmUrl = chrome.runtime.getURL("pkg/taieagi_bg.wasm");
        const resp = await fetch(wasmUrl);
        const bytes = await resp.arrayBuffer();
        await init({ module_or_path: bytes });
      } else {
        await init();
      }
      composer = new WasmComposer();
      if (typeof chrome !== "undefined" && chrome?.storage?.sync) {
        try {
          const stored = await chrome.storage.sync.get("outputMode");
          if (typeof stored?.outputMode === "number") {
            composer.set_output_mode(stored.outputMode);
          }
        } catch (err) {
          console.error("[Taieagi Background] Failed to load outputMode from storage:", err);
        }
      }
    })();
  }

  await initPromise;
  return composer!;
}

function snapshotState(comp: WasmComposer): ImeState {
  return {
    preedit: comp.preedit(),
    candidates: comp.candidates(),
    selected: comp.selected(),
    hints: comp.hints(),
  };
}

// Background service worker message listener
if (typeof chrome !== "undefined" && chrome?.runtime?.onMessage) {
  chrome.runtime.onMessage.addListener((message: BackgroundMessage, _sender, sendResponse) => {
    (async () => {
      try {
        const comp = await ensureInitialized();
        switch (message.type) {
          case "INIT": {
            sendResponse({
              ok: true,
              ...snapshotState(comp),
            });
            break;
          }
          case "PUSH_CHAR": {
            const handled = comp.push_char(message.char);
            sendResponse({
              ok: true,
              handled,
              ...snapshotState(comp),
            });
            break;
          }
          case "BACKSPACE": {
            const handled = comp.backspace();
            sendResponse({
              ok: true,
              handled,
              ...snapshotState(comp),
            });
            break;
          }
          case "CANCEL": {
            const handled = comp.cancel();
            sendResponse({
              ok: true,
              handled,
              ...snapshotState(comp),
            });
            break;
          }
          case "MOVE_SELECTION": {
            const handled = comp.move_selection(message.delta);
            sendResponse({
              ok: true,
              handled,
              ...snapshotState(comp),
            });
            break;
          }
          case "COMMIT_SELECTED": {
            const text = comp.commit_selected();
            sendResponse({
              ok: true,
              text: text ?? null,
              ...snapshotState(comp),
            });
            break;
          }
          case "COMMIT_INDEX": {
            const current = comp.selected();
            const delta = message.index - current;
            if (delta !== 0) {
              comp.move_selection(delta);
            }
            const text = comp.commit_selected();
            sendResponse({
              ok: true,
              text: text ?? null,
              ...snapshotState(comp),
            });
            break;
          }
          case "SET_OUTPUT_MODE": {
            comp.set_output_mode(message.mode);
            if (typeof chrome !== "undefined" && chrome?.storage?.sync) {
              chrome.storage.sync.set({ outputMode: message.mode }).catch((err) => {
                console.error("[Taieagi Background] Failed to save outputMode to storage:", err);
              });
            }
            sendResponse({ ok: true });
            break;
          }
          default:
            sendResponse({ ok: false, error: "Unknown message type" });
        }
      } catch (err: any) {
        console.error("[Taieagi Background] Error handling message:", err);
        sendResponse({ ok: false, error: String(err?.message ?? err) });
      }
    })();
    return true; // Keep message channel open for async response
  });
}
