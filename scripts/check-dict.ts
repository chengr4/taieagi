/**
 * Validates and canonically sorts human-maintained lexicon CSV files by lemma (詞目).
 *
 * Inspired by the corpus normalization and check gate workflow in
 * sysprog21/zhtw-mcp (https://github.com/sysprog21/zhtw-mcp, MIT License).
 */
const DICT_FILES = [
  "assets/supplement.csv",
  "assets/lyrics.csv",
  "assets/hints.csv",
  "assets/lyrics_hints.csv",
] as const;

interface CsvRow {
  lineNumber: number;
  lemma: string;
  raw: string;
}

function getLemma(line: string): string {
  const commaIdx = line.indexOf(",");
  return (commaIdx === -1 ? line : line.slice(0, commaIdx)).trim();
}

function compareCodepoints(a: string, b: string): number {
  return a < b ? -1 : a > b ? 1 : 0;
}

async function processCsvFile(path: string, writeMode: boolean): Promise<boolean> {
  const file = Bun.file(path);
  if (!(await file.exists())) {
    console.error(`❌ File not found: ${path}`);
    return false;
  }

  const text = await file.text();
  const lines = text.split(/\r?\n/).filter((l) => l.trim().length > 0);
  if (lines.length === 0) {
    console.error(`❌ Empty CSV file: ${path}`);
    return false;
  }

  const header = lines[0];
  const rows: CsvRow[] = lines.slice(1).map((raw, idx) => ({
    lineNumber: idx + 2,
    lemma: getLemma(raw),
    raw,
  }));

  let ok = true;
  const seenLemmas = new Map<string, number>();

  for (const row of rows) {
    if (row.lemma.length === 0) {
      console.error(`❌ ${path}:${row.lineNumber}: Empty lemma (詞目)`);
      ok = false;
      continue;
    }
    const prevLine = seenLemmas.get(row.lemma);
    if (prevLine !== undefined) {
      console.error(
        `❌ ${path}:${row.lineNumber}: Duplicate lemma "${row.lemma}" (first seen at line ${prevLine})`
      );
      ok = false;
    } else {
      seenLemmas.set(row.lemma, row.lineNumber);
    }
  }

  let isSorted = true;
  for (let i = 1; i < rows.length; i++) {
    if (compareCodepoints(rows[i - 1].lemma, rows[i].lemma) > 0) {
      isSorted = false;
      if (!writeMode) {
        console.error(
          `❌ ${path}:${rows[i].lineNumber}: Out of order — "${rows[i].lemma}" should appear before "${rows[i - 1].lemma}" (line ${rows[i - 1].lineNumber})`
        );
      }
      break;
    }
  }

  if (!isSorted) {
    if (writeMode && ok) {
      const sorted = [...rows].sort((a, b) => compareCodepoints(a.lemma, b.lemma));
      const updated = [header, ...sorted.map((r) => r.raw)].join("\n") + "\n";
      await Bun.write(path, updated);
      console.log(`✨ Sorted ${path} (${rows.length} entries)`);
    } else {
      ok = false;
    }
  } else if (ok) {
    console.log(`✓ ${path} is sorted (${rows.length} entries)`);
  }

  return ok;
}

const writeMode = process.argv.includes("--write");
let allOk = true;

for (const path of DICT_FILES) {
  const fileOk = await processCsvFile(path, writeMode);
  if (!fileOk) {
    allOk = false;
  }
}

if (!allOk) {
  if (!writeMode) {
    console.error("\n💡 Run `bun run dict:sort` to sort dictionary CSV files automatically.");
  }
  process.exit(1);
}
