// @vitest-environment jsdom
import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import { defineComponent, h, nextTick } from "vue";
import ConfirmDialog from "./ConfirmDialog.vue";

describe("ConfirmDialog", () => {
  it("focuses Cancel, cancels on Escape, and restores invoking focus", async () => {
    const invoker = document.createElement("button");
    document.body.append(invoker);
    invoker.focus();
    const wrapper = mount(ConfirmDialog, {
      attachTo: document.body,
      props: { open: false, title: "Delete?", message: "Cannot be undone." },
    });
    invoker.focus();
    await wrapper.setProps({ open: true });
    await nextTick();

    expect(document.activeElement).toBe(wrapper.find("button").element);
    await wrapper.find(".dialog").trigger("keydown", { key: "Escape" });
    expect(wrapper.emitted("cancel")).toHaveLength(1);

    await wrapper.setProps({ open: false });
    expect(document.activeElement).toBe(invoker);
    wrapper.unmount();
    invoker.remove();
  });

  it("describes each alert dialog with its own message element", () => {
    const wrapper = mount(defineComponent({
      setup: () => () => h("div", [
        h(ConfirmDialog, { open: true, title: "First", message: "First message" }),
        h(ConfirmDialog, { open: true, title: "Second", message: "Second message" }),
      ]),
    }));
    const dialogs = wrapper.findAll('[role="alertdialog"]');
    const firstDialog = dialogs[0];
    const secondDialog = dialogs[1];
    const firstMessageId = firstDialog.attributes("aria-describedby");
    const secondMessageId = secondDialog.attributes("aria-describedby");

    expect(firstMessageId).toBeTruthy();
    expect(secondMessageId).toBeTruthy();
    expect(firstMessageId).not.toBe(secondMessageId);
    expect(firstDialog.find(`#${firstMessageId}`).text()).toBe("First message");
    expect(secondDialog.find(`#${secondMessageId}`).text()).toBe("Second message");

    wrapper.unmount();
  });
});
