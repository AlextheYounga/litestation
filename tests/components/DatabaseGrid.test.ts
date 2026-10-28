// @vitest-environment jsdom
import { defineComponent } from "vue";
import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";
import DatabaseGrid from "../../src/components/DatabaseGrid.vue";
import type { BrowseRow } from "../../src/types/database";

vi.mock("ag-grid-vue3", () => ({
  AgGridVue: defineComponent({
    name: "AgGridVue",
    props: [
      "columnDefs",
      "defaultColDef",
      "rowData",
      "rowSelection",
    ],
    emits: ["selection-changed", "row-double-clicked"],
    template: "<div />",
  }),
}));

const row: BrowseRow = {
  identity: { kind: "rowid", value: "1" },
  values: [{ kind: "text", value: "Ada" }],
};

describe("DatabaseGrid", () => {
  it("disables client-side sorting and exposes selected database rows", async () => {
    const wrapper = mount(DatabaseGrid, {
      props: { columns: ["name"], rows: [row], offset: 0 },
    });
    const grid = wrapper.findComponent({ name: "AgGridVue" });

    expect(grid.props("defaultColDef").sortable).toBe(false);
    expect(grid.props("columnDefs")[0].sortable).toBeUndefined();

    const gridRow = grid.props("rowData")[0];
    await grid.vm.$emit("selection-changed", {
      selectedNodes: [{ data: gridRow }],
    });

    expect(wrapper.emitted("selection-change")).toEqual([[[row]]]);
  });
});
