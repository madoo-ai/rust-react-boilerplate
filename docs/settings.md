# Settings

The Settings area lives under `/settings` and requires a session. The layout renders one `PageHeader` and a left navigation for Profile, Password, Appearance and Security.

## Profile

`/settings/profile` updates:

- name
- email
- phone
- job title
- department

Changing the email checks uniqueness, sets `emailVerified` to false and tries to send a verification email. If SMTP is unavailable, the update still succeeds and the link is logged.

The page also contains the danger zone.

## Password

`/settings/password` calls `POST /api/auth/change-password`:

- current password is required,
- new password is 8-128 characters,
- confirmation must match,
- other sessions are revoked.

On success the form resets and shows a toast.

## Appearance

`/settings/appearance` contains:

- theme: light, dark or system
- color theme: the Stripe-inspired default and 37 OpenCode palettes
- language: Bahasa Indonesia or English

The theme is stored in a `theme` cookie and applied by the inline script in `web/index.html` before paint. The provider updates the class and the cookie.

The color theme is stored in a `color-theme` cookie. Each OpenCode palette has a light block and a dark block in `web/src/styles/themes.css`, so the mode and the palette stay independent. The default palette lives in `:root` and `.dark` in `web/src/styles/globals.css`.

The palette data comes from `web/src/lib/theme/opencode-themes.json`. The CSS and the picker metadata are generated:

```bash
make themes
```

Do not edit `web/src/styles/themes.css` or `web/src/lib/theme/theme-options.ts` by hand.

Language is stored in a `locale` cookie. The SPA updates the catalogue immediately. The server default is `APP_LOCALE`.

## Security

`/settings/security` enables and disables TOTP two-factor authentication.

Enabling:

1. Confirm the account password.
2. Receive a TOTP URI and ten backup codes.
3. Scan the QR code.
4. Enter a six-digit code to finish.

Disabling also requires the password. The page refreshes the session after either action so the status badge and the session user reflect the change.

## Account deletion

The danger zone on the Profile page deletes the current account.

- It requires a recent password confirmation. If the 15-minute cookie is missing, the page links to `/confirm-password?next=/settings/profile`.
- The last superadmin cannot delete their account.
- The deletion cascades sessions, accounts, 2FA records and other user relations.
- The client clears the session and redirects to `/login?deleted=1`.

## SPA note

The theme cookie is applied before the first paint. The locale cookie is read by the SPA at boot; there is no server-rendered HTML to translate. Keep the default locale usable and avoid layout shifts when the catalogue changes. `WEB_URL` is the public web origin used in verification and reset links. In development it points to Vite on port 5173. In production it defaults to `APP_URL`.
