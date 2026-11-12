import { expect, test } from "@playwright/test";

import {
  deleteUsersByEmail,
  loginAs,
  logout,
  setRegistrationApproval,
  uniqueEmail,
} from "./helpers";

const email = uniqueEmail("e2e-pending");
const password = "password123";

test.beforeAll(async ({ request }) => {
  await setRegistrationApproval(request, false);
});

test.afterAll(async ({ request }) => {
  await deleteUsersByEmail(request, [email]);
  await setRegistrationApproval(request, false);
});

test("registration approval gates a new account until an admin verifies it", async ({
  page,
}) => {
  await loginAs(page);
  await page.goto("/admin/users");

  const toggle = page.getByRole("switch", {
    name: "Wajibkan persetujuan",
  });
  await expect(toggle).not.toBeChecked();
  await toggle.click();
  await expect(
    page.getByText("Persetujuan pendaftaran diperbarui."),
  ).toBeVisible();

  await logout(page);
  await page.goto("/register");

  await page.getByLabel("Nama").fill("E2E Pending User");
  await page.getByLabel("Email").fill(email);
  await page.getByLabel("Kata sandi", { exact: true }).fill(password);
  await page.getByLabel("Konfirmasi kata sandi").fill(password);
  await page.getByRole("button", { name: "Buat akun" }).click();

  await expect(page).toHaveURL(/\/login\?error=account_pending/);
  await expect(page.getByText(/menunggu persetujuan/i)).toBeVisible();

  await page.getByLabel("Email").fill(email);
  await page.getByLabel("Kata sandi", { exact: true }).fill(password);
  await page.getByRole("button", { name: "Masuk", exact: true }).click();
  await expect(page.getByText(/menunggu persetujuan/i)).toBeVisible();

  await loginAs(page);
  await page.goto("/admin/users");

  const row = page.getByRole("row", { name: new RegExp(email) });
  await expect(row).toBeVisible();
  await row.getByRole("button", { name: "Buka menu" }).click();
  await page.getByRole("menuitem", { name: "Verifikasi" }).click();
  await expect(page.getByText("Pengguna diverifikasi.")).toBeVisible();
  await expect(
    page.getByRole("row", { name: new RegExp(email) }),
  ).toContainText("Aktif");

  await logout(page);
  await loginAs(page, email, password);
  await expect(
    page.getByRole("heading", { name: "Dasbor", exact: true }),
  ).toBeVisible();
});
