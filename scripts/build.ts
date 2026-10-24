import { spawnSync } from "node:child_process";

console.log("🔨 [1/3] Compiling Rust core to wasm32-unknown-unknown (release)...");
const cargoBuild = spawnSync(
  "cargo",
  ["build", "--target", "wasm32-unknown-unknown", "--release"],
  { stdio: "inherit" }
);

if (cargoBuild.status !== 0) {
  console.error("❌ Cargo build failed!");
  process.exit(cargoBuild.status ?? 1);
}

console.log("📦 [2/3] Generating TypeScript & WebAssembly bindings into extension/pkg...");
const wasmBindgen = spawnSync(
  "wasm-bindgen",
  [
    "target/wasm32-unknown-unknown/release/taieagi.wasm",
    "--out-dir",
    "extension/pkg",
    "--target",
    "web",
  ],
  { stdio: "inherit" }
);

if (wasmBindgen.status !== 0) {
  console.error("❌ wasm-bindgen failed!");
  process.exit(wasmBindgen.status ?? 1);
}

console.log("🎨 [3/3] Bundling extension scripts (content.ts & background.ts)...");
const scriptsBuild = await Bun.build({
  entrypoints: ["extension/src/content.ts", "extension/src/background.ts"],
  outdir: "extension",
  naming: "[name].[ext]",
  target: "browser",
  minify: false,
});

if (!scriptsBuild.success) {
  console.error("❌ Extension scripts build failed!", scriptsBuild.logs);
  process.exit(1);
}

// Sanitize import.meta for Chrome extension classic script / worker contexts
for (const file of [
  "extension/content.js",
  "extension/background.js",
  "extension/pkg/taieagi.js",
]) {
  if (await Bun.file(file).exists()) {
    const text = await Bun.file(file).text();
    if (text.includes("import.meta")) {
      const sanitized = text.replaceAll(
        "import.meta.url",
        'chrome.runtime?.getURL?.("") ?? ""'
      );
      await Bun.write(file, sanitized);
      console.log(`🧹 Sanitized import.meta in ${file}`);
    }
  }
}

console.log("✨ Build succeeded! Extension ready in ./extension");
