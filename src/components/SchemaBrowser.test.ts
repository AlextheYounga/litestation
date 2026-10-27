// @vitest-environment jsdom
import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import SchemaBrowser from "./SchemaBrowser.vue";

describe("SchemaBrowser", () => {
  it("keeps indexes out of the database-wide object browser", () => {
    const wrapper = mount(SchemaBrowser, {
      props: {
        schema: {
          tables: [{ name: "users", tableName: "users", sql: null }],
          views: [],
          indexes: [{ name: "users_email", tableName: "users", sql: null }],
          triggers: [],
        },
      },
    });

    expect(wrapper.text()).toContain("users");
    expect(wrapper.text()).not.toContain("users_email");
    expect(wrapper.text()).not.toContain("Indexes");
  });
});
