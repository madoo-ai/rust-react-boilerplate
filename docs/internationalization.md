# Internationalization

## Source of truth

The `locale` cookie is the single source of truth. There is no `/[locale]` segment and no locale prefix in URLs.

- `web/src/lib/i18n.tsx` defines locales, the default (`id`), labels and the cookie name.
- `I18nProvider` reads the cookie, loads `web/src/messages/<locale>.json` and exposes `useTranslations`.
- `useLocale` and `useSetLocale` read and write the cookie.
- The `<html lang>` attribute follows the active locale.
- If the cookie is missing or invalid, the default locale is Indonesian. The browser language is not used automatically.

## Message files

`web/src/messages/en.json` and `web/src/messages/id.json` share the same key tree. Sections:

- `common`, `nav`, `auth`, `validation`, `errors`
- `dashboard`, `users`, `roles`, `settings`
- `permissions`, `preferences`, `errorPages`

Use nested sections and section-scoped keys. Do not use dots inside a key. The dot is the path separator.

## Using translations

```tsx
const t = useTranslations("users");
t("title");
```

```tsx
const t = useTranslations();
t("errors.unexpected");
```

Interpolation uses braces:

```json
"showing": "Showing {from}-{to} of {total}"
```

```tsx
t("showing", { from: 1, to: 10, total: 42 });
```

## Validation and errors

Zod schemas use message keys:

```ts
z.string().min(8, "validation.passwordLength");
```

Forms translate them with `translateFieldError`. API errors carry a message key. `errorMessage` translates the key and falls back to the server text.

## Adding a locale

1. Add the locale to `locales` in `web/src/lib/i18n.tsx`.
2. Add a label to `localeLabels`.
3. Copy `web/src/messages/en.json` to the new locale and translate every value.
4. Add the locale to `APP_LOCALE` validation in `api/src/xmodule/config.rs`.
5. Run `npm run check:i18n`.
6. Add a test or screenshot for the locale switch if the layout changes.

## Checks

`npm run check:i18n` compares the key sets and fails on missing keys, extra keys or empty values. It is part of `make lint`.

## Time zone

The dashboard groups sign-ups by Jakarta day (`Asia/Jakarta`). Change `APP_TIME_ZONE` to move the bucket boundary. The HTML and JSON dates stay in UTC. The browser formats them for the active locale.
