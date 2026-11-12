import { expect, test } from "@playwright/test";

import { loginAs } from "./helpers";

test("the public landing page previews themes and links to the demo", async ({
  page,
}) => {
  await page.goto("/");

  await expect(
    page.getByRole("heading", {
      level: 1,
      name: /Mulai dari bagian/,
    }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "Buka demo" })).toBeVisible();

  await page.getByRole("button", { name: "Dracula" }).first().click();
  await expect(page.locator("html")).toHaveAttribute(
    "data-color-theme",
    "dracula",
  );

  await page.getByRole("link", { name: "Buka demo" }).click();
  await expect(page).toHaveURL(/\/login$/);
});

test("signed-in visitors see the dashboard link", async ({ page }) => {
  await loginAs(page);
  await page.goto("/");

  await expect(
    page.getByRole("link", { name: "Buka dasbor" }).first(),
  ).toBeVisible();
});
