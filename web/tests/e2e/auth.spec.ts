import { expect, test } from "@playwright/test";

import { deleteUsersByEmail, loginAs, uniqueEmail } from "./helpers";

const openRegistrationEmail = uniqueEmail("e2e-open");

test.afterAll(async ({ request }) => {
  await deleteUsersByEmail(request, [openRegistrationEmail]);
});

test("guests are redirected to the login page", async ({ page }) => {
  await page.goto("/admin/users");
  await expect(page).toHaveURL(/\/login\?next=%2Fadmin%2Fusers/);
  await expect(
    page.getByRole("heading", { name: "Masuk", exact: true }),
  ).toBeVisible();
});

test("superadmin signs in and sees the team dashboard", async ({ page }) => {
  await loginAs(page);
  await expect(
    page.getByRole("heading", { name: "Dasbor", exact: true }),
  ).toBeVisible();
  await expect(page.getByText("Total pengguna")).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Peran", exact: true }),
  ).toBeVisible();
});

test("pending users are blocked with a clear message", async ({ page }) => {
  await page.goto("/login");
  await page.getByLabel("Email").fill("nabila@example.com");
  await page.getByLabel("Kata sandi", { exact: true }).fill("password");
  await page.getByRole("button", { name: "Masuk", exact: true }).click();
  await expect(page.getByText(/menunggu persetujuan/i)).toBeVisible();
});

test("HR sees users but not role management", async ({ page }) => {
  await loginAs(page, "hr@example.com");
  await expect(
    page.getByRole("link", { name: "Pengguna", exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Peran", exact: true }),
  ).toHaveCount(0);
});

test("registration signs in immediately when approval is off", async ({
  page,
}) => {
  await page.goto("/register");
  await page.getByLabel("Nama").fill("E2E Open User");
  await page.getByLabel("Email").fill(openRegistrationEmail);
  await page.getByLabel("Kata sandi", { exact: true }).fill("password123");
  await page.getByLabel("Konfirmasi kata sandi").fill("password123");
  await page.getByRole("button", { name: "Buat akun" }).click();

  await expect(page).toHaveURL("/dashboard");
  await expect(
    page.getByRole("heading", { name: "Dasbor", exact: true }),
  ).toBeVisible();
});

test("regular users see a personal dashboard", async ({ page }) => {
  await loginAs(page, "user@example.com");
  await expect(
    page.getByRole("heading", { name: "Dasbor", exact: true }),
  ).toBeVisible();
  await expect(page.getByText("Total pengguna")).toHaveCount(0);
  await expect(page.getByText("Regular User").first()).toBeVisible();
});
