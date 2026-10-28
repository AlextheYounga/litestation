/* oxlint-disable typescript/prefer-readonly-parameter-types */

import { collectSourceFiles, getRelativePath, readFile } from './_utils.ts';
import { describe, it, strictEqual } from './_test.ts';

const MAX_LINES = 400;
const NO_FILES = 0;

describe('file-too-long', () => {
  const files = collectSourceFiles(['src', 'app', 'lib'], ['.ts', '.tsx']);
  if (files.length === NO_FILES) return;

  const violations: string[] = [];

  for (const file of files) {
    const code = readFile(file);
    const parts = code.split('\n');
    const lineCount = parts[parts.length - 1] === '' ? parts.length - 1 : parts.length;
    if (lineCount > MAX_LINES) {
      violations.push(`  ${getRelativePath(file)}: ${lineCount} lines (max ${MAX_LINES})`);
    }
  }

  it('file length', () => {
    strictEqual(
      violations.length,
      0,
      `File(s) exceed ${MAX_LINES} lines:\n${violations.join('\n')}`,
    );
  });
});
