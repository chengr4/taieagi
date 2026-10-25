import { readFileSync } from "node:fs";
import { join } from "node:path";
import init, { WasmComposer } from "../extension/pkg/taieagi.js";

async function main() {
  console.log("Loading WebAssembly module...");
  const wasmPath = join(import.meta.dir, "..", "extension", "pkg", "taieagi_bg.wasm");
  const wasmBytes = readFileSync(wasmPath);
  await init({ module_or_path: wasmBytes });

  const composer = new WasmComposer();

  // Test push letters
  for (const c of "tsui") {
    const handled = composer.push_char(c);
    if (!handled) {
      throw new Error(`Expected true (handled) for '${c}', got ${handled}`);
    }
  }

  if (composer.preedit() !== "tsui") {
    throw new Error(`Expected preedit 'tsui', got '${composer.preedit()}'`);
  }

  const cands = composer.candidates();
  console.log("Candidates for 'tsui':", cands.slice(0, 5));
  if (!cands.includes("水")) {
    throw new Error(`Expected candidates to include '水', got: ${JSON.stringify(cands)}`);
  }

  // Test commit index
  const committedText = composer.commit_index(composer.selected());
  console.log("Committed text:", committedText);
  if (committedText !== cands[0]) {
    throw new Error(`Expected '${cands[0]}', got '${committedText}'`);
  }

  if (composer.preedit() !== "") {
    throw new Error("Preedit should be cleared after commit");
  }

  // Test Option A: Japanese loanword with Kana conversion
  for (const c of "nagasi") {
    composer.push_char(c);
  }
  const nagasiCands = composer.candidates();
  console.log("Candidates for 'nagasi':", nagasiCands);
  if (!nagasiCands.includes("那卡西")) {
    throw new Error(`Expected 'nagasi' to match '那卡西', got: ${JSON.stringify(nagasiCands)}`);
  }
  composer.cancel();

  console.log("All WASM runtime assertions passed successfully in Bun!");
}

main().catch((err) => {
  console.error("Test failed:", err);
  process.exit(1);
});
