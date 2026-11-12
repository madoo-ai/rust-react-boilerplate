import { describe, expect, it } from "vitest";

import {
  adminNavItems,
  filterNavItems,
  shouldShowAdminGroup,
  type NavUser,
} from "@/config/navigation";

const superadmin: NavUser = {
  permissions: ["view_admin", "manage_users", "manage_roles"],
  role: "superadmin",
};

const hr: NavUser = {
  permissions: ["view_admin", "manage_users"],
  role: "hr",
};

const user: NavUser = {
  permissions: [],
  role: "user",
};

describe("navigation gating", () => {
  it("shows both admin links to a superadmin", () => {
    expect(filterNavItems(adminNavItems, superadmin)).toHaveLength(2);
    expect(shouldShowAdminGroup(superadmin)).toBe(true);
  });

  it("hides role management from HR", () => {
    const items = filterNavItems(adminNavItems, hr);
    expect(items.map((item) => item.titleKey)).toEqual(["users"]);
    expect(shouldShowAdminGroup(hr)).toBe(true);
  });

  it("hides the admin group from a regular user", () => {
    expect(filterNavItems(adminNavItems, user)).toHaveLength(0);
    expect(shouldShowAdminGroup(user)).toBe(false);
  });
});
