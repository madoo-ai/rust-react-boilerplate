import { describe, expect, it } from "vitest";

import en from "@/messages/en.json";
import id from "@/messages/id.json";

type Messages = Record<string, unknown>;

function flattenKeys(value: Messages, prefix = ""): string[] {
  return Object.entries(value).flatMap(([key, child]) => {
    const path = prefix ? `${prefix}.${key}` : key;
    if (child && typeof child === "object" && !Array.isArray(child)) {
      return flattenKeys(child as Messages, path);
    }
    return [path];
  });
}

describe("message catalogues", () => {
  it("have identical key sets", () => {
    const enKeys = new Set(flattenKeys(en));
    const idKeys = new Set(flattenKeys(id));

    expect([...enKeys].sort()).toEqual([...idKeys].sort());
  });

  it("have no empty messages", () => {
    for (const catalogue of [en, id]) {
      for (const key of flattenKeys(catalogue)) {
        const value = key.split(".").reduce<unknown>((current, segment) => {
          if (!current || typeof current !== "object") return undefined;
          return (current as Messages)[segment];
        }, catalogue);
        expect(typeof value).toBe("string");
        expect((value as string).trim().length).toBeGreaterThan(0);
      }
    }
  });
});
