// @vitest-environment jsdom
import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";
import DatabaseTools from "./DatabaseTools.vue";

const services = vi.hoisted(() => ({
  chooseSqlDump: vi.fn(),
  chooseSqlDumpDestination: vi.fn(),
  chooseSqliteFile: vi.fn(),
  chooseSqliteCopyDestination: vi.fn(),
  exportSqlDump: vi.fn(),
  importSqlDump: vi.fn(),
  importSqliteFile: vi.fn(),
  normalizeBackendError: (error: unknown) => ({ code: "error", message: String(error) }),
}));

vi.mock("../services/database", () => services);

describe("DatabaseTools", () => {
  it("enables SQL import for every open connection", () => {
    const wrapper = mount(DatabaseTools, { props: { connectionId: 1 } });
    expect(wrapper.get("button").text()).toBe("Import SQL dump");
    expect(wrapper.get("button").attributes("disabled")).toBeUndefined();
  });

  it("treats a cancelled SQL dump picker as a neutral action", async () => {
    services.chooseSqlDump.mockResolvedValueOnce(null);
    const wrapper = mount(DatabaseTools, { props: { connectionId: 1 } });

    await wrapper.get("button").trigger("click");

    expect(wrapper.findComponent({ name: "ConfirmDialog" }).props("open")).toBe(false);
    expect(wrapper.find("[role=alert]").exists()).toBe(false);
  });

  it("does not show a SQL confirmation picked for a previous connection", async () => {
    let resolvePicker!: (path: string | null) => void;
    services.chooseSqlDump.mockReturnValueOnce(new Promise((resolve) => { resolvePicker = resolve; }));
    const wrapper = mount(DatabaseTools, { props: { connectionId: 1 } });

    await wrapper.get("button").trigger("click");
    await wrapper.setProps({ connectionId: 2 });
    resolvePicker("/old-connection.sql");
    await wrapper.vm.$nextTick();

    expect(wrapper.findComponent({ name: "ConfirmDialog" }).props("open")).toBe(false);
    expect(wrapper.find("[role=alert]").exists()).toBe(false);
  });

  it("releases busy state when a copy completes after switching connections", async () => {
    let resolveCopy!: (result: { path: string }) => void;
    services.chooseSqliteFile.mockResolvedValueOnce("/source.sqlite");
    services.chooseSqliteCopyDestination.mockResolvedValueOnce("/copy.sqlite");
    services.importSqliteFile.mockReturnValueOnce(new Promise((resolve) => { resolveCopy = resolve; }));
    const wrapper = mount(DatabaseTools, { props: { connectionId: 1 } });

    await wrapper.findAll("button")[1].trigger("click");
    await wrapper.vm.$nextTick();
    expect(wrapper.findAll("button")[1].attributes("disabled")).toBeDefined();

    await wrapper.setProps({ connectionId: 2 });
    expect(wrapper.findAll("button")[1].attributes("disabled")).toBeUndefined();
    expect(wrapper.emitted("busy-change")).toContainEqual([false]);

    resolveCopy({ path: "/copy.sqlite" });
    await wrapper.vm.$nextTick();
    expect(wrapper.find(".status-message").exists()).toBe(false);
  });
});
