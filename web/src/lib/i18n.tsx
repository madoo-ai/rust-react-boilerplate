import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";

import en from "@/messages/en.json";
import id from "@/messages/id.json";

export const locales = ["id", "en"] as const;
export type Locale = (typeof locales)[number];

export const defaultLocale: Locale = "id";
export const localeCookieName = "locale";

export const localeLabels: Record<Locale, string> = {
  id: "Bahasa Indonesia",
  en: "English",
};

export function isLocale(value: string | undefined | null): value is Locale {
  return !!value && locales.includes(value as Locale);
}

type MessageTree = Record<string, unknown>;
type MessagesByLocale = Record<Locale, MessageTree>;

const catalogues: MessagesByLocale = {
  en: en as MessageTree,
  id: id as MessageTree,
};

type Variables = Record<string, string | number>;

type I18nContextValue = {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (key: string, variables?: Variables) => string;
};

const I18nContext = createContext<I18nContextValue | null>(null);

function readCookie(name: string): string | undefined {
  if (typeof document === "undefined") return undefined;

  const match = document.cookie.match(
    new RegExp(
      `(?:^|; )${name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}=([^;]*)`,
    ),
  );

  return match ? decodeURIComponent(match[1]) : undefined;
}

function writeCookie(name: string, value: string) {
  document.cookie = `${name}=${encodeURIComponent(value)};path=/;max-age=31536000;samesite=lax`;
}

function lookup(tree: MessageTree, key: string): string | undefined {
  const value = key.split(".").reduce<unknown>((current, segment) => {
    if (!current || typeof current !== "object") return undefined;
    return (current as MessageTree)[segment];
  }, tree);

  return typeof value === "string" ? value : undefined;
}

function interpolate(message: string, variables?: Variables): string {
  if (!variables) return message;

  return message.replace(/\{(\w+)\}/g, (match, name: string) => {
    const value = variables[name];
    return value === undefined ? match : String(value);
  });
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(() => {
    const cookie = readCookie(localeCookieName);
    return isLocale(cookie) ? cookie : defaultLocale;
  });

  const t = useCallback(
    (key: string, variables?: Variables) => {
      const message =
        lookup(catalogues[locale], key) ??
        lookup(catalogues[defaultLocale], key);

      return interpolate(message ?? key, variables);
    },
    [locale],
  );

  const setLocale = useCallback((next: Locale) => {
    setLocaleState(next);
    writeCookie(localeCookieName, next);
  }, []);

  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);

  const value = useMemo(
    () => ({ locale, setLocale, t }),
    [locale, setLocale, t],
  );

  return <I18nContext value={value}>{children}</I18nContext>;
}

function useI18n() {
  const context = useContext(I18nContext);

  if (!context) {
    throw new Error("useI18n must be used within an I18nProvider");
  }

  return context;
}

export function useLocale(): Locale {
  return useI18n().locale;
}

export function useSetLocale() {
  return useI18n().setLocale;
}

export function useTranslations(namespace?: string) {
  const { t } = useI18n();

  return useCallback(
    (key: string, variables?: Variables) =>
      t(namespace ? `${namespace}.${key}` : key, variables),
    [namespace, t],
  );
}

export function useTranslation() {
  return useI18n();
}

export function translateFieldError(
  message: string | undefined,
  t: (key: string) => string,
): string | undefined {
  if (!message) return undefined;
  if (message.startsWith("validation.")) {
    return t(message.replace("validation.", ""));
  }

  return message;
}
