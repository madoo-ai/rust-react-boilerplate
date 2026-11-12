import { expect, type APIRequestContext, type Page } from "@playwright/test";

export async function loginAs(
  page: Page,
  email = "superadmin@example.com",
  password = "password",
) {
  await page.goto("/login");
  await page.getByLabel("Email").fill(email);
  await page.getByLabel("Kata sandi", { exact: true }).fill(password);
  await page.getByRole("button", { name: "Masuk", exact: true }).click();
  await page.waitForURL("/dashboard");
}

export async function logout(page: Page) {
  await page.locator('[data-sidebar="footer"] button').first().click();
  await page.getByRole("menuitem", { name: "Keluar" }).click();
  await expect(page).toHaveURL(/\/login/);
}

export function uniqueEmail(prefix = "e2e") {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}@example.com`;
}

async function apiLogin(request: APIRequestContext) {
  const response = await request.post("/api/auth/sign-in/email", {
    data: {
      email: "superadmin@example.com",
      password: "password",
      rememberMe: true,
    },
  });

  expect(response.ok()).toBeTruthy();
}

export async function deleteUsersByEmail(
  request: APIRequestContext,
  emails: string[],
) {
  await apiLogin(request);

  for (const email of emails) {
    const response = await request.get(
      `/api/users?search=${encodeURIComponent(email)}`,
    );
    const body = (await response.json()) as { users: { id: string }[] };

    for (const user of body.users) {
      await request.delete(`/api/users/${user.id}`);
    }
  }
}

export async function deleteRoleByName(
  request: APIRequestContext,
  name: string,
) {
  await apiLogin(request);
  const response = await request.get("/api/roles");
  const body = (await response.json()) as {
    roles: { id: string; name: string }[];
  };

  for (const role of body.roles.filter((item) => item.name === name)) {
    await request.delete(`/api/roles/${role.id}`);
  }
}

export async function setRegistrationApproval(
  request: APIRequestContext,
  enabled: boolean,
) {
  await apiLogin(request);
  await request.put("/api/users/registration-approval", { data: { enabled } });
}

export async function resetSuperadminName(request: APIRequestContext) {
  await apiLogin(request);
  const list = await request.get("/api/users?search=superadmin%40example.com");
  const body = (await list.json()) as { users: { id: string }[] };
  const id = body.users[0]?.id;

  if (!id) return;

  const detail = await request.get(`/api/users/${id}`);
  const data = (await detail.json()) as {
    user: {
      name: string;
      email: string;
      phoneNumber: string | null;
      jobTitle: string | null;
      department: string | null;
      role: string | null;
      status: string;
    };
  };

  await request.patch(`/api/users/${id}`, {
    data: {
      name: "Super Admin",
      email: data.user.email,
      phone: data.user.phoneNumber ?? "",
      jobTitle: data.user.jobTitle ?? "",
      department: data.user.department ?? "",
      role: data.user.role,
      status: data.user.status,
      password: "",
    },
  });
}
