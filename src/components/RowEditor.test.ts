// @vitest-environment jsdom
import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import RowEditor from "./RowEditor.vue";
import type { TableColumn } from "../types/database";

const columns: TableColumn[] = [
  {
    name: "id",
    declaredType: "INTEGER",
    notNull: true,
    defaultSql: null,
    primaryKeyPosition: 1,
  },
  {
    name: "name",
    declaredType: "TEXT",
    notNull: false,
    defaultSql: null,
    primaryKeyPosition: 0,
  },
];

describe("RowEditor", () => {
  it("rebuilds values when editing a different row", async () => {
    const wrapper = mount(RowEditor, {
      props: {
        columns,
        primaryKey: ["id"],
        initial: [
          { kind: "integer", value: "1" },
          { kind: "text", value: "row A" },
        ],
      },
    });

    await wrapper.setProps({
      initial: [
        { kind: "integer", value: "2" },
        { kind: "text", value: "row B" },
      ],
    });
    await wrapper.find("form").trigger("submit");

    expect(wrapper.emitted("save")?.[0][0]).toEqual([
      { column: "id", value: { kind: "integer", value: "2" } },
      { column: "name", value: { kind: "text", value: "row B" } },
    ]);
  });
});
