/* oxlint-disable typescript/prefer-readonly-parameter-types */

import { collectSourceFiles, getRelativePath, readFile } from './_utils.ts';
import { describe, it, strictEqual } from './_test.ts';

const NO_FILES = 0;
const SRC_DIRS = ['src', 'app', 'lib'];
const EXTENSIONS = ['.ts', '.tsx'];
const NOISE_VALUES = new Set(['.', '{}/', '{}']);

const PATH_SUFFIXES = ['.gpg', '.service', '.sh', '.socket', '.toml'];

function stringLiterals(line: string): string[] {
  const re = /"([^"\\]|\\.)*"|'([^'\\]|\\.)*'|`([^`\\]|\\.)*`/g;
  const values: string[] = [];
  for (const match of line.matchAll(re)) {
    const raw = match[0];
    const delim = raw[0];
    const inner = raw.slice(1, -1).replace(/\\(.)/g, '$1');
    if (delim === '`' && inner.includes('${')) continue;
    values.push(inner);
  }
  return values;
}

function isPathLike(v: string): boolean {
  return v.includes('/')
    || v.startsWith('.')
    || PATH_SUFFIXES.some((s) => v.endsWith(s));
}

function isEnvVarLike(v: string): boolean {
  return v.length >= 3
    && /^[A-Z][A-Z0-9_]*$/.test(v);
}

function isConfigLike(v: string): boolean {
  return v.includes('_')
    && /^[a-z][a-z0-9_]*$/.test(v);
}

type StateKind = 'path' | 'env' | 'config';

function classify(v: string): StateKind | null {
  if (isPathLike(v)) return 'path';
  if (isEnvVarLike(v)) return 'env';
  if (isConfigLike(v)) return 'config';
  return null;
}

interface HitEntry {
  value: string;
  kind: StateKind;
  hits: string[];
}

describe('no-duplicated-state', () => {
  const files = collectSourceFiles(SRC_DIRS, EXTENSIONS);
  if (files.length === NO_FILES) return;

  const seen = new Map<string, HitEntry>();

  for (const file of files) {
    const code = readFile(file);
    const rel = getRelativePath(file);

    for (const [i, line] of code.split('\n').entries()) {
      if (line.trimStart().startsWith('import ')) continue;
      if (line.includes('process.env.')) continue;
      for (const value of stringLiterals(line)) {
        const kind = classify(value);
        if (!kind || NOISE_VALUES.has(value)) continue;

        const key = `${kind}:${value}`;
        if (!seen.has(key)) seen.set(key, { value, kind, hits: [] });
        seen.get(key)!.hits.push(`${rel}:${i + 1} ${line.trim()}`);
      }
    }
  }

  const violations: string[] = [];
  for (const [, entry] of seen) {
    if (entry.hits.length < 2) continue;
    const fileSet = new Set(entry.hits.map((h) => h.split(':')[0]));
    if (fileSet.size < 2) continue;
    violations.push(`  ${entry.kind} "${entry.value}": ${entry.hits.join('; ')}`);
  }

  it('duplicated state', () => {
    strictEqual(
      violations.length,
      0,
      `Duplicated state literals found. Centralize intentional state:\n${violations.join('\n')}`,
    );
  });
});
