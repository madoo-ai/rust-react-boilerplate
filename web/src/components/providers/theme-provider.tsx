import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";

import {
  colorThemeCookieName,
  defaultColorTheme,
  defaultTheme,
  isColorTheme,
  isTheme,
  themeCookieName,
  type Theme,
} from "@/lib/theme";
import type { ColorThemeId } from "@/lib/theme/theme-options";

type ThemeContextValue = {
  theme: Theme;
  colorTheme: ColorThemeId;
  setTheme: (theme: Theme) => void;
  setColorTheme: (theme: ColorThemeId) => void;
};

const ThemeContext = createContext<ThemeContextValue | null>(null);

function readCookie(name: string): string | undefined {
  const match = document.cookie.match(
    new RegExp(
      `(?:^|; )${name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}=([^;]*)`,
    ),
  );

  return match ? decodeURIComponent(match[1]) : undefined;
}

function resolveTheme(theme: Theme): "light" | "dark" {
  if (theme === "system") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  }

  return theme;
}

function applyTheme(theme: Theme) {
  const resolved = resolveTheme(theme);
  const root = document.documentElement;

  root.classList.toggle("dark", resolved === "dark");
  root.style.colorScheme = resolved;
}

function persistTheme(theme: Theme) {
  document.cookie = `${themeCookieName}=${theme};path=/;max-age=31536000;samesite=lax`;
}

function applyColorTheme(theme: ColorThemeId) {
  if (theme === "default") {
    delete document.documentElement.dataset.colorTheme;
    return;
  }

  document.documentElement.dataset.colorTheme = theme;
}

function persistColorTheme(theme: ColorThemeId) {
  document.cookie = `${colorThemeCookieName}=${theme};path=/;max-age=31536000;samesite=lax`;
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<Theme>(() => {
    const cookie = readCookie(themeCookieName);
    return isTheme(cookie) ? cookie : defaultTheme;
  });
  const [colorTheme, setColorThemeState] = useState<ColorThemeId>(() => {
    const cookie = readCookie(colorThemeCookieName);
    return isColorTheme(cookie) ? cookie : defaultColorTheme;
  });

  useEffect(() => {
    applyTheme(theme);
    applyColorTheme(colorTheme);

    if (theme !== "system") return;

    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => applyTheme("system");

    media.addEventListener("change", onChange);
    return () => media.removeEventListener("change", onChange);
  }, [theme, colorTheme]);

  const setTheme = useCallback((nextTheme: Theme) => {
    setThemeState(nextTheme);
    persistTheme(nextTheme);
    applyTheme(nextTheme);
  }, []);

  const setColorTheme = useCallback((nextTheme: ColorThemeId) => {
    setColorThemeState(nextTheme);
    persistColorTheme(nextTheme);
    applyColorTheme(nextTheme);
  }, []);

  const value = useMemo(
    () => ({ theme, colorTheme, setTheme, setColorTheme }),
    [theme, colorTheme, setTheme, setColorTheme],
  );

  return <ThemeContext value={value}>{children}</ThemeContext>;
}

export function useTheme() {
  const context = useContext(ThemeContext);

  if (!context) {
    throw new Error("useTheme must be used within a ThemeProvider");
  }

  return context;
}
