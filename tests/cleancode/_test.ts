/* oxlint-disable import/no-unassigned-import, import/no-named-export */

import './node.d.ts';

import { describe as nodeDescribe, it as nodeIt } from 'node:test';
import assert from 'node:assert';

const describe = nodeDescribe;
const it = nodeIt;
const strictEqual = assert.strictEqual.bind(assert);

export { describe, it, strictEqual };
