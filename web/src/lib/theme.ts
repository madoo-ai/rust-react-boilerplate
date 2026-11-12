import {
  COLOR_THEME_OPTIONS,
  type ColorThemeId,
} from "@/lib/theme/theme-options";

export const themeModes = ["light", "dark", "system"] as const;
export type Theme = (typeof themeModes)[number];

export const themeCookieName = "theme";
export const defaultTheme: Theme = "system";

export const colorThemeCookieName = "color-theme";
export const defaultColorTheme: ColorThemeId = "default";

export function isTheme(value: string | undefined | null): value is Theme {
  return !!value && themeModes.includes(value as Theme);
}

export function isColorTheme(
  value: string | undefined | null,
): value is ColorThemeId {
  return !!value && COLOR_THEME_OPTIONS.some((theme) => theme.id === value);
}
