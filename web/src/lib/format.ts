import type { Locale } from "@/lib/i18n";

const localeTags: Record<Locale, string> = {
  id: "id-ID",
  en: "en-US",
};

export function formatDate(value: string | Date, locale: Locale): string {
  const date = typeof value === "string" ? new Date(value) : value;

  return new Intl.DateTimeFormat(localeTags[locale], {
    dateStyle: "medium",
  }).format(date);
}

export function formatNumber(value: number, locale: Locale): string {
  return new Intl.NumberFormat(localeTags[locale]).format(value);
}

export function chartDateLabel(value: string, locale: Locale): string {
  const date = new Date(`${value}T00:00:00`);

  return new Intl.DateTimeFormat(localeTags[locale], {
    day: "numeric",
    month: "short",
  }).format(date);
}

export function initials(name: string): string {
  return name
    .split(/\s+/)
    .filter(Boolean)
    .map((part) => part[0])
    .join("")
    .slice(0, 2)
    .toUpperCase();
}
