export interface StatementRange {
  start: number;
  end: number;
  text: string;
}

const potentiallyDestructiveKeywords = new Set([
  "INSERT",
  "UPDATE",
  "DELETE",
  "REPLACE",
  "CREATE",
  "ALTER",
  "DROP",
  "VACUUM",
  "REINDEX",
  // A PRAGMA may be either a read or a write. Warn conservatively because
  // distinguishing every writable PRAGMA is more error-prone than useful.
  "PRAGMA",
]);

/** Finds SQL statement boundaries without treating quoted/comment semicolons as separators. */
export function statements(sql: string): StatementRange[] {
  const ranges: StatementRange[] = [];
  let start = 0;
  let quote = "";
  let line = false;
  let block = false;
  for (let i = 0; i < sql.length; i += 1) {
    const ch = sql[i];
    const next = sql[i + 1];
    if (line) {
      if (ch === "\n") line = false;
      continue;
    }
    if (block) {
      if (ch === "*" && next === "/") {
        block = false;
        i += 1;
      }
      continue;
    }
    if (quote) {
      if (quote === "]" ? ch === "]" : ch === quote) {
        if (next === quote && quote !== "]") i += 1;
        else quote = "";
      }
      continue;
    }
    if (ch === "-" && next === "-") {
      line = true;
      i += 1;
      continue;
    }
    if (ch === "/" && next === "*") {
      block = true;
      i += 1;
      continue;
    }
    if (ch === "'" || ch === '"' || ch === "`") {
      quote = ch;
      continue;
    }
    if (ch === "[") {
      quote = "]";
      continue;
    }
    if (ch === ";") {
      const text = sql.slice(start, i + 1);
      if (meaningful(text)) ranges.push({ start, end: i + 1, text });
      start = i + 1;
    }
  }
  const text = sql.slice(start);
  if (meaningful(text)) ranges.push({ start, end: sql.length, text });
  return ranges;
}

function meaningful(text: string): boolean {
  return text.replace(/--[^\n]*|\/\*[\s\S]*?\*\//g, "").trim().length > 0;
}

/** Returns the first SQL keyword, ignoring only leading whitespace/comments. */
export function firstMeaningfulKeyword(sql: string): string | null {
  let index = 0;
  while (index < sql.length) {
    while (/\s/.test(sql[index] ?? "")) index += 1;
    if (sql.startsWith("--", index)) {
      const end = sql.indexOf("\n", index + 2);
      index = end === -1 ? sql.length : end + 1;
      continue;
    }
    if (sql.startsWith("/*", index)) {
      const end = sql.indexOf("*/", index + 2);
      index = end === -1 ? sql.length : end + 2;
      continue;
    }
    break;
  }

  const match = /^[A-Za-z]+/.exec(sql.slice(index));
  return match?.[0].toUpperCase() ?? null;
}

/** User-facing warning classification; backend enforcement remains authoritative. */
export function isPotentiallyDestructive(sql: string): boolean {
  const keyword = firstMeaningfulKeyword(sql);
  return keyword !== null && potentiallyDestructiveKeywords.has(keyword);
}

export function statementAtCursor(sql: string, cursor: number): string | null {
  const found = statements(sql).find(
    (range) =>
      (cursor >= range.start && cursor < range.end) ||
      (cursor === sql.length && range.end === sql.length),
  );
  return found?.text ?? null;
}
export function runAllStatement(sql: string): string | null {
  const found = statements(sql);
  return found.length === 1 ? found[0].text : null;
}
