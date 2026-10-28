/* oxlint-disable import/no-unassigned-import, import/no-named-export, typescript/prefer-readonly-parameter-types, typescript/no-unsafe-type-assertion */

import './node.d.ts';

import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';
import { parse } from '@babel/parser';

const IGNORE_DIRS = new Set(['node_modules', 'dist', 'build', 'coverage', '.next', 'out', 'vendor']);

interface NodeLocation {
  readonly start: {
    readonly line: number;
  };
}

interface AstNodeBase {
  readonly type: string;
  readonly start: number;
  readonly end: number;
  readonly loc: NodeLocation;
  readonly [key: string]: unknown;
}

interface ProgramNode extends AstNodeBase {
  type: 'Program';
  readonly body: readonly AstNode[];
}

interface FileNode extends AstNodeBase {
  type: 'File';
  readonly program: ProgramNode;
}

interface BlockStatementNode extends AstNodeBase {
  type: 'BlockStatement';
  readonly body: readonly AstNode[];
}

interface CatchClauseNode extends AstNodeBase {
  type: 'CatchClause';
  readonly body: BlockStatementNode;
}

interface TryStatementNode extends AstNodeBase {
  type: 'TryStatement';
  readonly handler?: CatchClauseNode | null;
}

interface CallExpressionNode extends AstNodeBase {
  type: 'CallExpression';
  readonly callee: AstNode;
  readonly arguments: readonly AstNode[];
}

interface MemberExpressionNode extends AstNodeBase {
  type: 'MemberExpression';
  readonly computed: boolean;
  readonly object: AstNode;
  readonly property: AstNode;
}

interface ReturnStatementNode extends AstNodeBase {
  type: 'ReturnStatement';
  readonly argument?: AstNode | null;
}

interface ThrowStatementNode extends AstNodeBase {
  type: 'ThrowStatement';
  readonly argument: AstNode;
}

interface AwaitExpressionNode extends AstNodeBase {
  type: 'AwaitExpression';
  readonly argument: AstNode;
}

interface LogicalExpressionNode extends AstNodeBase {
  type: 'LogicalExpression';
  readonly operator: '??' | '||';
  readonly left: AstNode;
  readonly right: AstNode;
}

interface StringLiteralNode extends AstNodeBase {
  type: 'StringLiteral';
  readonly value: string;
}

interface NumericLiteralNode extends AstNodeBase {
  type: 'NumericLiteral';
  readonly value: number;
}

interface BooleanLiteralNode extends AstNodeBase {
  type: 'BooleanLiteral';
  readonly value: boolean;
}

interface BigIntLiteralNode extends AstNodeBase {
  type: 'BigIntLiteral';
  readonly value: bigint;
}

interface RegExpLiteralNode extends AstNodeBase {
  type: 'RegExpLiteral';
}

interface ArrayExpressionNode extends AstNodeBase {
  type: 'ArrayExpression';
}

interface ObjectExpressionNode extends AstNodeBase {
  type: 'ObjectExpression';
}

interface FunctionExpressionNode extends AstNodeBase {
  type: 'FunctionExpression';
  readonly body: BlockStatementNode;
}

interface ArrowFunctionExpressionNode extends AstNodeBase {
  type: 'ArrowFunctionExpression';
  readonly body: BlockStatementNode | AstNode;
}

interface ClassExpressionNode extends AstNodeBase {
  type: 'ClassExpression';
}

interface NewExpressionNode extends AstNodeBase {
  type: 'NewExpression';
}

interface TemplateLiteralNode extends AstNodeBase {
  type: 'TemplateLiteral';
}

interface IdentifierNode extends AstNodeBase {
  type: 'Identifier';
  readonly name: string;
}

type AstNode =
  | FileNode
  | ProgramNode
  | BlockStatementNode
  | CatchClauseNode
  | TryStatementNode
  | CallExpressionNode
  | MemberExpressionNode
  | ReturnStatementNode
  | ThrowStatementNode
  | AwaitExpressionNode
  | LogicalExpressionNode
  | StringLiteralNode
  | NumericLiteralNode
  | BooleanLiteralNode
  | BigIntLiteralNode
  | RegExpLiteralNode
  | ArrayExpressionNode
  | ObjectExpressionNode
  | FunctionExpressionNode
  | ArrowFunctionExpressionNode
  | ClassExpressionNode
  | NewExpressionNode
  | TemplateLiteralNode
  | IdentifierNode;

function walkFiles(dir: string, files: string[], extensions: readonly string[]): void {
  if (!existsSync(dir)) return;
  for (const entry of readdirSync(dir)) {
    const fullPath = join(dir, entry);
    if (statSync(fullPath).isDirectory()) {
      if (!IGNORE_DIRS.has(entry)) {
        walkFiles(fullPath, files, extensions);
      }
      continue;
    }

    if (extensions.some((ext) => entry.endsWith(ext))) {
      files.push(fullPath);
    }
  }
}

function collectSourceFiles(srcDirs: readonly string[], extensions: readonly string[]): string[] {
  const files: string[] = [];
  for (const srcDir of srcDirs) {
    walkFiles(join(process.cwd(), srcDir), files, extensions);
  }
  return files;
}

function parseSource(code: string, parserPlugins: readonly string[] = ['jsx', 'typescript']): AstNode {
  return parse(code, { sourceType: 'unambiguous', plugins: parserPlugins }) as AstNode;
}

function tryParseSource(code: string, parserPlugins: readonly string[] = ['jsx', 'typescript']): AstNode | null {
  try {
    return parseSource(code, parserPlugins);
  } catch {
    return null;
  }
}

function readFile(filePath: string): string {
  return readFileSync(filePath, 'utf-8');
}

function getRelativePath(filePath: string): string {
  return relative(process.cwd(), filePath);
}

function isAstNode(value: unknown): value is AstNodeBase {
  return Boolean(value)
    && typeof value === 'object'
    && 'type' in value
    && typeof (value as { type?: unknown }).type === 'string';
}

function walkArrayChildren(
  children: readonly unknown[],
  visitor: (node: Readonly<AstNode>) => void,
  walkNode: (node: unknown, visitor: (node: Readonly<AstNode>) => void) => void,
): void {
  for (const item of children) {
    if (isAstNode(item)) {
      walkNode(item, visitor);
    }
  }
}

function walkAST(node: unknown, visitor: (node: Readonly<AstNode>) => void): void {
  if (!isAstNode(node)) return;
  visitor(node as Readonly<AstNode>);
  for (const key of Object.keys(node)) {
    const child = node[key];
    if (Array.isArray(child)) {
      walkArrayChildren(child, visitor, walkAST);
      continue;
    }
    if (isAstNode(child)) {
      walkAST(child, visitor);
    }
  }
}

export {
  collectSourceFiles,
  getRelativePath,
  parseSource,
  readFile,
  tryParseSource,
  walkAST,
};
