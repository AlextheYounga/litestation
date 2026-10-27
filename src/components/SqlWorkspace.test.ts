// @vitest-environment jsdom
import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";
import SqlWorkspace from "./SqlWorkspace.vue";

const services = vi.hoisted(() => ({
  chooseExportDestination: vi.fn().mockResolvedValue("/tmp/result.json"),
  executeSql: vi.fn().mockResolvedValue({ columns: ["name"], rows: [[{ kind: "text", value: "Ada" }]], affectedRows: null, truncated: true, durationMs: 1, schemaChanged: false }),
  exportRows: vi.fn().mockResolvedValue({ path: "/tmp/result.json", rowCount: 1 }),
  normalizeBackendError: (error: unknown) => ({ code: "error", message: String(error) }),
}));

vi.mock("../services/database", () => services);

describe("SqlWorkspace", () => {
  it("exposes the editor source through its v-model contract", async () => {
    const wrapper = mount(SqlWorkspace, {
      props: { connectionId: 1, modelValue: "SELECT 1;" },
    });
    const editor = wrapper.findComponent({ name: "SqlEditor" });

    await editor.vm.$emit("update:modelValue", "SELECT 2;");

    expect(wrapper.emitted("update:modelValue")).toEqual([["SELECT 2;"]]);
    wrapper.unmount();
  });

  it("exports exactly the displayed bounded columns and rows", async () => {
    const wrapper = mount(SqlWorkspace, { props: { connectionId: 1, modelValue: "SELECT name FROM users;" } });
    await wrapper.findComponent({ name: "SqlEditor" }).vm.$emit("run", "", 0);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Export JSON"));
    const exportJson = wrapper.findAll("button").find((button) => button.text() === "Export JSON");
    expect(exportJson).toBeDefined();
    await exportJson!.trigger("click");

    expect(services.exportRows).toHaveBeenCalledWith({
      columns: ["name"],
      rows: [[{ kind: "text", value: "Ada" }]],
      format: "json",
      destinationPath: "/tmp/result.json",
    });
    wrapper.unmount();
  });

  it("confirms a destructive direct run before calling the backend", async () => {
    services.executeSql.mockClear();
    const wrapper = mount(SqlWorkspace, { props: { connectionId: 1, modelValue: "UPDATE users SET name = 'Ada';" } });
    await wrapper.findComponent({ name: "SqlEditor" }).vm.$emit("run", "UPDATE users SET name = 'Ada';", 0);

    expect(services.executeSql).not.toHaveBeenCalled();
    expect(wrapper.text()).toContain("may change schema or data");
    await wrapper.get("button.danger").trigger("click");
    await vi.waitFor(() => expect(services.executeSql).toHaveBeenCalledTimes(1));
    expect(services.executeSql).toHaveBeenCalledWith({ connectionId: 1, sql: "UPDATE users SET name = 'Ada';", maxRows: 100 });
    wrapper.unmount();
  });

  it("does not execute a destructive direct run when cancelled", async () => {
    services.executeSql.mockClear();
    const wrapper = mount(SqlWorkspace, { props: { connectionId: 1, modelValue: "DELETE FROM users;" } });
    await wrapper.findComponent({ name: "SqlEditor" }).vm.$emit("run", "DELETE FROM users;", 0);
    const cancel = wrapper.findAll("button").find((button) => button.text() === "Cancel");
    await cancel!.trigger("click");
    expect(services.executeSql).not.toHaveBeenCalled();
    expect(wrapper.text()).not.toContain("may change schema or data");
    wrapper.unmount();
  });
});
