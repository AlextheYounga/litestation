// @vitest-environment jsdom
import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import TableIndexes from "./TableIndexes.vue";

describe("TableIndexes", () => {
  it("shows only indexes belonging to the selected table", () => {
    const wrapper = mount(TableIndexes, {
      props: {
        tableName: "users",
        indexes: [
          { name: "users_email", tableName: "users", sql: "CREATE INDEX users_email ON users(email)" },
          { name: "orders_date", tableName: "orders", sql: "CREATE INDEX orders_date ON orders(created_at)" },
        ],
      },
    });

    expect(wrapper.text()).toContain("users_email");
    expect(wrapper.text()).not.toContain("orders_date");
  });
});
