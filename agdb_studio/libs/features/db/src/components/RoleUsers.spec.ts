import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import type { DbUser, DbUserRole } from "@agnesoft/agdb_api";
import RoleUsers from "./RoleUsers.vue";

const user = (username: string, role: DbUserRole): DbUser => ({
  username,
  role,
});
const USERS = [
  user("owner", "admin"),
  user("ann", "write"),
  user("bob", "read"),
];

const mountUsers = (
  props: Partial<{ users: DbUser[] | undefined; canEdit: boolean }> = {},
) =>
  mount(RoleUsers, {
    props: {
      title: "Users",
      users: USERS,
      owner: "owner",
      canEdit: true,
      ...props,
    },
  });

type Wrapper = ReturnType<typeof mountUsers>;
const usernames = (wrapper: Wrapper) =>
  wrapper.findAll(".username").map((u) => u.text());

describe("RoleUsers", () => {
  it("shows nothing while the users load", () => {
    const wrapper = mountUsers({ users: undefined });
    expect(wrapper.find("h2").text()).toBe("Users");
    expect(wrapper.findAll(".role-group")).toHaveLength(0);
    expect(wrapper.findAll(".role-count")).toHaveLength(0);
  });

  it("groups users by role with counts", () => {
    const wrapper = mountUsers();
    expect(wrapper.findAll(".role-group-header").map((h) => h.text())).toEqual([
      "Admin (1)",
      "Read/Write (1)",
      "Read Only (1)",
    ]);
    expect(wrapper.findAll(".role-count").map((c) => c.text())).toEqual([
      "1 Admin",
      "1 Read/Write",
      "1 Read Only",
    ]);
  });

  it("filters users, keeping the full counts", async () => {
    const wrapper = mountUsers();
    await wrapper.find(".search-input").setValue(" ANN ");
    expect(usernames(wrapper)).toEqual(["ann"]);
    expect(wrapper.findAll(".role-count")).toHaveLength(3);
    await wrapper.find(".search-input").setValue("");
    expect(usernames(wrapper)).toEqual(["owner", "ann", "bob"]);
  });

  it("filters nothing while the users load", async () => {
    const wrapper = mountUsers({ users: undefined });
    await wrapper.find(".search-input").setValue("ann");
    expect(usernames(wrapper)).toEqual([]);
  });

  it("lets editors add, change and remove users but not the owner", async () => {
    const wrapper = mountUsers();
    await wrapper.find(".add-button").trigger("click");
    expect(wrapper.emitted("add")).toHaveLength(1);

    const names = wrapper.findAll(".username");
    expect(names[0]!.classes()).not.toContain("clickable");
    expect(names[1]!.classes()).toContain("clickable");
    await names[0]!.trigger("click");
    expect(wrapper.emitted("edit")).toBeUndefined();
    await names[1]!.trigger("click");
    expect(wrapper.emitted("edit")).toEqual([["ann", "write"]]);

    const removes = wrapper.findAll(".remove-button");
    expect(removes).toHaveLength(2);
    await removes[1]!.trigger("click");
    expect(wrapper.emitted("remove")).toEqual([["bob"]]);
  });

  it("shows read only users to others", async () => {
    const wrapper = mountUsers({ canEdit: false });
    expect(wrapper.find(".add-button").exists()).toBe(false);
    expect(wrapper.find(".remove-button").exists()).toBe(false);
    expect(wrapper.find(".clickable").exists()).toBe(false);
    const names = wrapper.findAll(".username");
    expect(names).toHaveLength(3);
    for (const name of names) await name.trigger("click");
    expect(wrapper.emitted("edit")).toBeUndefined();
  });

  it("collapses long groups and expands them", async () => {
    const many = Array.from({ length: 12 }, (_, i) => user(`u${i}`, "read"));
    const wrapper = mountUsers({ users: many });
    const toggle = () => wrapper.find(".show-more-button");
    expect(usernames(wrapper)).toHaveLength(10);
    expect(toggle().text()).toBe("+2 more");
    await toggle().trigger("click");
    expect(usernames(wrapper)).toHaveLength(12);
    expect(toggle().text()).toBe("Show less");
    await toggle().trigger("click");
    expect(usernames(wrapper)).toHaveLength(10);
  });
});
