"use client";

import {
  CheckIcon as Check,
  Translate01Icon as Languages,
} from "@x1nx3r/altics-react-icons";

import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  isLocale,
  localeLabels,
  locales,
  useLocale,
  useSetLocale,
  useTranslations,
  type Locale,
} from "@/lib/i18n";

export function LocaleSwitcher() {
  const t = useTranslations("preferences");
  const currentLocale = useLocale();
  const setLocale = useSetLocale();
  const activeLocale = isLocale(currentLocale) ? currentLocale : "id";

  function handleSelect(locale: Locale) {
    if (locale === activeLocale) return;
    setLocale(locale);
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          size="sm"
          className="gap-2"
          aria-label={t("language")}
        >
          <Languages />
          <span className="hidden sm:inline">{localeLabels[activeLocale]}</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="min-w-40">
        {locales.map((locale) => (
          <DropdownMenuItem key={locale} onSelect={() => handleSelect(locale)}>
            <span className="flex-1">{localeLabels[locale]}</span>
            {activeLocale === locale ? <Check className="size-4" /> : null}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
