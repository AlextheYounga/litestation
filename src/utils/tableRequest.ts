export interface TableRequestKey {
  connectionId: number;
  tableName: string;
  offset: number;
}

/** Keeps only the latest table-page request eligible to change UI state. */
export class TableRequestGuard {
  private generation = 0;

  begin(key: TableRequestKey): number {
    // Keep the key in this API: callers must deliberately identify what they load.
    void key;
    this.generation += 1;
    return this.generation;
  }

  isCurrent(generation: number): boolean {
    return generation === this.generation;
  }

  invalidate(): void {
    this.generation += 1;
  }
}
