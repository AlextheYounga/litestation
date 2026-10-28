declare module 'node:test' {
  export function describe(name: string, fn: () => void): void;
  export function it(name: string, fn: () => void): void;
}

declare module 'node:fs' {
  export function existsSync(path: string): boolean;
  export function readdirSync(path: string): string[];
  export function readFileSync(path: string, encoding: 'utf-8'): string;
  export function statSync(path: string): { isDirectory(): boolean };
}

declare module 'node:path' {
  export function join(...parts: string[]): string;
  export function relative(from: string, to: string): string;
}

declare const process: {
  cwd(): string;
};

declare module 'node:assert' {
  const assert: {
    strictEqual(actual: unknown, expected: unknown, message?: string): void;
  };

  export default assert;
}
