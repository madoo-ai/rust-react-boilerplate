import { expect, test } from "@playwright/test";

import { deleteUsersByEmail, loginAs, uniqueEmail } from "./helpers";

const email = uniqueEmail("e2e-user");
const updatedName = "E2E User Updated";

test.afterAll(async ({ request }) => {
  await deleteUsersByEmail(request, [email]);
});

test("superadmin can create, edit and delete a user", async ({ page }) => {
  await loginAs(page);
  await page.goto("/admin/users");

  await page.getByRole("link", { name: "Buat pengguna" }).click();
  await expect(
    page.getByRole("heading", { name: "Buat pengguna", exact: true }),
  ).toBeVisible();

  await page.getByLabel("Nama").fill("E2E User");
  await page.getByLabel("Email").fill(email);
  await page.getByLabel("Telepon").fill("08123456789");
  await page.getByLabel("Jabatan").fill("QA Engineer");
  await page.getByLabel("Departemen").fill("Engineering");
  await page.getByLabel("Kata sandi", { exact: true }).fill("password123");
  await page.getByRole("button", { name: "Buat pengguna" }).click();

  await expect(page).toHaveURL(/\/admin\/users$/);
  await expect(page.getByText("Pengguna dibuat.")).toBeVisible();

  const row = page.getByRole("row", { name: new RegExp(email) });
  await expect(row).toBeVisible();
  await expect(row.getByText("E2E User")).toBeVisible();

  await row.getByRole("button", { name: "Buka menu" }).click();
  await page.getByRole("menuitem", { name: "Ubah" }).click();
  await expect(page).toHaveURL(/\/edit$/);

  await page.getByLabel("Nama").fill(updatedName);
  await page.getByRole("button", { name: "Simpan perubahan" }).click();

  await expect(page).toHaveURL(/\/admin\/users$/);
  await expect(page.getByText("Pengguna diperbarui.")).toBeVisible();

  const updatedRow = page.getByRole("row", { name: new RegExp(email) });
  await expect(updatedRow.getByText(updatedName)).toBeVisible();

  await updatedRow.getByRole("button", { name: "Buka menu" }).click();
  await page.getByRole("menuitem", { name: "Hapus" }).click();

  const dialog = page.getByRole("alertdialog");
  await expect(dialog).toBeVisible();
  await dialog.getByRole("button", { name: "Hapus pengguna" }).click();

  await expect(page.getByText("Pengguna dihapus.")).toBeVisible();
  await expect(page.getByText(email)).toHaveCount(0);
});
