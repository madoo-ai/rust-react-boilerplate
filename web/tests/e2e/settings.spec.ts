import { expect, test } from "@playwright/test";

import { loginAs, resetSuperadminName } from "./helpers";

test.beforeAll(async ({ request }) => {
  await resetSuperadminName(request);
});

test.afterAll(async ({ request }) => {
  await resetSuperadminName(request);
});

test("profile changes persist and can be reverted", async ({ page }) => {
  await loginAs(page);
  await page.goto("/settings/profile");

  const nameInput = page.getByLabel("Nama").first();
  const original = await nameInput.inputValue();
  const updated = `${original} E2E`;

  await nameInput.fill(updated);
  await page.getByRole("button", { name: "Simpan profil" }).click();
  await expect(page.getByText("Profil diperbarui.")).toBeVisible();
  await expect(page.locator('[data-sidebar="footer"]')).toContainText(updated);

  await nameInput.fill(original);
  await page.getByRole("button", { name: "Simpan profil" }).click();
  await expect(page.locator('[data-sidebar="footer"]')).toContainText(original);
});

test("locale can be switched from the settings page", async ({ page }) => {
  await loginAs(page);
  await page.goto("/settings/appearance");

  await page.getByRole("combobox", { name: "Bahasa" }).click();
  await page.getByRole("option", { name: "English" }).click();
  await expect(
    page.getByRole("heading", { name: "Settings", exact: true }),
  ).toBeVisible();
  await expect(
    page.getByText("Appearance", { exact: true }).first(),
  ).toBeVisible();

  await page.getByRole("combobox", { name: "Language" }).click();
  await page.getByRole("option", { name: "Bahasa Indonesia" }).click();
  await expect(
    page.getByRole("heading", { name: "Pengaturan", exact: true }),
  ).toBeVisible();
});

test("theme can be switched to dark and back", async ({ page }) => {
  await loginAs(page);
  await page.goto("/settings/appearance");

  await page.getByRole("radio", { name: "Gelap" }).click();
  await expect(page.locator("html")).toHaveClass(/dark/);

  await page.getByRole("radio", { name: "Sistem" }).click();
  await expect(page.locator("html")).not.toHaveClass(/dark/);
});

test("color theme applies its light and dark palettes", async ({ page }) => {
  await loginAs(page);
  await page.goto("/settings/appearance");

  await page.getByRole("button", { name: /Dracula/i }).click();
  await expect(page.locator("html")).toHaveAttribute(
    "data-color-theme",
    "dracula",
  );

  const lightBackground = await page.evaluate(() =>
    getComputedStyle(document.documentElement)
      .getPropertyValue("--background")
      .trim(),
  );
  expect(lightBackground).toBe("#f8f8f2");

  await page.getByRole("radio", { name: "Gelap" }).click();
  const darkBackground = await page.evaluate(() =>
    getComputedStyle(document.documentElement)
      .getPropertyValue("--background")
      .trim(),
  );
  expect(darkBackground).toBe("#1d1e28");

  await page.reload();
  await expect(page.locator("html")).toHaveAttribute(
    "data-color-theme",
    "dracula",
  );
});
