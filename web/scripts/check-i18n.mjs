import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const locales = ["en", "id"];

function readMessages(locale) {
  const path = resolve(process.cwd(), "src/messages", `${locale}.json`);
  return JSON.parse(readFileSync(path, "utf8"));
}

function flattenKeys(value, prefix = "") {
  return Object.entries(value).flatMap(([key, child]) => {
    const path = prefix ? `${prefix}.${key}` : key;

    if (child && typeof child === "object" && !Array.isArray(child)) {
      return flattenKeys(child, path);
    }

    return [path];
  });
}

function getValue(messages, key) {
  return key.split(".").reduce((current, segment) => {
    if (!current || typeof current !== "object") return undefined;
    return current[segment];
  }, messages);
}

const catalogues = Object.fromEntries(
  locales.map((locale) => [locale, readMessages(locale)]),
);

const allKeys = new Set(
  locales.flatMap((locale) => flattenKeys(catalogues[locale])),
);

let failed = false;

for (const locale of locales) {
  const keys = new Set(flattenKeys(catalogues[locale]));
  const missing = [...allKeys].filter((key) => !keys.has(key));

  if (missing.length > 0) {
    failed = true;
    console.error(`[${locale}] missing keys:`);
    for (const key of missing.sort()) {
      console.error(`  - ${key}`);
    }
  }

  for (const key of keys) {
    const value = getValue(catalogues[locale], key);

    if (typeof value !== "string" || value.trim().length === 0) {
      failed = true;
      console.error(`[${locale}] empty or non-string message: ${key}`);
    }
  }
}

if (failed) {
  process.exitCode = 1;
} else {
  console.log(`i18n OK: ${allKeys.size} keys across ${locales.join(", ")}.`);
}
