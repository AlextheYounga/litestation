import { describe, expect, it } from "vitest";
import { TableRequestGuard } from "./tableRequest";

describe("TableRequestGuard", () => {
  it("does not apply a stale mocked browse response after selecting another table", async () => {
    const guard = new TableRequestGuard();
    const first = guard.begin({
      connectionId: 1,
      tableName: "authors",
      offset: 0,
    });
    const second = guard.begin({
      connectionId: 1,
      tableName: "books",
      offset: 0,
    });

    const responses = [Promise.resolve("authors"), Promise.resolve("books")];
    let displayed = "";
    const apply = async (generation: number, response: Promise<string>) => {
      const table = await response;
      if (guard.isCurrent(generation)) displayed = table;
    };

    await apply(second, responses[1]);
    await apply(first, responses[0]);

    expect(guard.isCurrent(first)).toBe(false);
    expect(guard.isCurrent(second)).toBe(true);
    expect(displayed).toBe("books");
  });
});
