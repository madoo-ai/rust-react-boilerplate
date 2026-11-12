import { expect, test } from "@playwright/test";

import { deleteRoleByName, loginAs } from "./helpers";

const roleName = `e2e_role_${Date.now()}`;
const roleLabel = "E2E Role";
const updatedLabel = "E2E Role Updated";

test.afterAll(async ({ request }) => {
  await deleteRoleByName(request, roleName);
});

test("superadmin can create, edit and delete a role", async ({ page }) => {
  await loginAs(page);
  await page.goto("/admin/roles");

  await page.getByRole("button", { name: "Buat peran" }).click();
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();

  await dialog.getByLabel("Nama").fill(roleName);
  await dialog.getByLabel("Label").fill(roleLabel);
  await dialog.getByLabel("Deskripsi").fill("Created by the E2E suite.");
  await dialog.getByLabel("Kelola pengguna").check();
  await dialog.getByRole("button", { name: "Buat peran" }).click();

  await expect(page.getByText("Peran dibuat.")).toBeVisible();

  const row = page.getByRole("row", { name: new RegExp(roleName) });
  await expect(row).toBeVisible();
  await expect(row.getByText(roleLabel)).toBeVisible();

  await row.getByRole("button", { name: "Ubah" }).click();
  const editDialog = page.getByRole("dialog");
  await expect(editDialog).toBeVisible();
  await editDialog.getByLabel("Label").fill(updatedLabel);
  await editDialog.getByRole("button", { name: "Simpan perubahan" }).click();

  await expect(page.getByText("Peran diperbarui.")).toBeVisible();

  const updatedRow = page.getByRole("row", { name: new RegExp(roleName) });
  await expect(updatedRow.getByText(updatedLabel)).toBeVisible();

  await updatedRow.getByRole("button", { name: "Hapus" }).click();
  const alert = page.getByRole("alertdialog");
  await expect(alert).toBeVisible();
  await alert.getByRole("button", { name: "Hapus peran" }).click();

  await expect(page.getByText("Peran dihapus.")).toBeVisible();
  await expect(page.getByText(roleName)).toHaveCount(0);
});
