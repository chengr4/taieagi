import { spawnSync } from "node:child_process";

const type = process.argv[2]; // "patch" | "minor"

if (type === "major") {
  console.error("❌ Major releases are disabled during the 0.x development phase. Use 'patch' or 'minor'.");
  process.exit(1);
}

if (type !== "patch" && type !== "minor") {
  console.error("❌ Usage: bun run bump <patch|minor>");
  process.exit(1);
}

// 1. Read current version from package.json
const pkg = await Bun.file("package.json").json();
const parts = pkg.version.split(".").map(Number);
if (parts.length !== 3 || parts.some(isNaN)) {
  console.error(`❌ Invalid semver format in package.json: ${pkg.version}`);
  process.exit(1);
}
const [major, minor, patch] = parts;

// 2. Compute next version
const nextVersion =
  type === "patch"
    ? `${major}.${minor}.${patch + 1}`
    : `${major}.${minor + 1}.0`;

console.log(`🚀 Bumping version: ${pkg.version} -> ${nextVersion} (${type})`);

// 3. Update package.json
pkg.version = nextVersion;
await Bun.write("package.json", JSON.stringify(pkg, null, 2) + "\n");
console.log("  ✓ Updated package.json");

// 4. Update Cargo.toml
const cargoToml = await Bun.file("Cargo.toml").text();
const updatedCargo = cargoToml.replace(
  /^version\s*=\s*"[^"]+"/m,
  `version = "${nextVersion}"`
);
await Bun.write("Cargo.toml", updatedCargo);
console.log("  ✓ Updated Cargo.toml");

// 5. Update Cargo.lock
const cargoCheck = spawnSync("cargo", ["check", "--quiet"], { stdio: "inherit" });
if (cargoCheck.status !== 0) {
  console.error("❌ Failed to update Cargo.lock via cargo check");
  process.exit(cargoCheck.status ?? 1);
}
console.log("  ✓ Updated Cargo.lock");

// 6. Build extension and sync manifest.json
const buildRun = spawnSync("bun", ["run", "scripts/build.ts"], { stdio: "inherit" });
if (buildRun.status !== 0) {
  console.error("❌ Failed to build extension and sync manifest.json");
  process.exit(buildRun.status ?? 1);
}

console.log(`\n🎉 Successfully bumped version to ${nextVersion}!`);
