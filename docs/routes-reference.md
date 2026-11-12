# Routes reference

## Public web routes

| Path                        | Purpose                       |
| --------------------------- | ----------------------------- |
| `/`                         | Public landing page           |
| `/login`                    | Sign in                       |
| `/register`                 | Self-registration             |
| `/forgot-password`          | Request a reset link          |
| `/reset-password?token=...` | Choose a new password         |
| `/verify-email?token=...`   | Verification result page      |
| `/two-factor`               | TOTP or backup-code challenge |
| `/403`                      | Forbidden page                |
| `*`                         | Not found page                |

The landing page is public and shows a dashboard link when a valid session exists. `/confirm-password` requires a session.

## Authenticated web routes

| Path                         | Access          | Purpose                                                          |
| ---------------------------- | --------------- | ---------------------------------------------------------------- |
| `/dashboard`                 | any active user | Dashboard (team view with `view_admin`, personal view otherwise) |
| `/settings`                  | any active user | Redirect to `/settings/profile`                                  |
| `/settings/profile`          | any active user | Profile form and danger zone                                     |
| `/settings/password`         | any active user | Change password                                                  |
| `/settings/appearance`       | any active user | Theme, color theme and locale                                    |
| `/settings/security`         | any active user | Enable or disable 2FA                                            |
| `/confirm-password?next=...` | any active user | Recent password confirmation                                     |

## Admin web routes

| Path                     | Access         | Purpose                             |
| ------------------------ | -------------- | ----------------------------------- |
| `/admin/users`           | `manage_users` | User list, filters, approval switch |
| `/admin/users/create`    | `manage_users` | Create user                         |
| `/admin/users/:id/edit`  | `manage_users` | Edit user                           |
| `/admin/roles`           | `manage_roles` | Role and permission management      |

## API routes

All API routes return JSON. Unknown `/api/*` paths return the JSON 404 envelope.

### Auth

| Method | Path                                  | Access         | Purpose                              |
| ------ | ------------------------------------- | -------------- | ------------------------------------ |
| GET    | `/api/auth/session`                   | public         | Current session or `null`            |
| POST   | `/api/auth/sign-up/email`             | public         | Create an account                    |
| POST   | `/api/auth/sign-in/email`             | public         | Password sign-in or 2FA challenge    |
| POST   | `/api/auth/sign-out`                  | session        | Revoke the current session           |
| POST   | `/api/auth/verify-email`              | public         | Consume a verification token         |
| POST   | `/api/auth/send-verification-email`   | session        | Resend verification mail             |
| POST   | `/api/auth/forgot-password`           | public         | Request a reset link                 |
| POST   | `/api/auth/reset-password`            | public         | Reset with a token                   |
| POST   | `/api/auth/change-password`           | session        | Change the password                  |
| POST   | `/api/auth/confirm-password`          | session        | Set the 15-minute confirmation cookie |
| GET    | `/api/auth/sessions`                  | session        | List sessions                        |
| DELETE | `/api/auth/sessions/{id}`             | session        | Revoke one session                   |
| POST   | `/api/auth/sessions/revoke-others`    | session        | Revoke other sessions                 |
| POST   | `/api/auth/two-factor/enable`         | session        | Start TOTP enrollment                |
| POST   | `/api/auth/two-factor/verify`         | session        | Finish TOTP enrollment               |
| POST   | `/api/auth/two-factor/disable`        | session        | Disable 2FA                          |
| POST   | `/api/auth/two-factor/backup-codes`   | session        | Regenerate backup codes              |
| POST   | `/api/auth/two-factor/verify-totp`    | challenge      | Verify the sign-in TOTP code         |
| POST   | `/api/auth/two-factor/verify-backup-code` | challenge  | Verify a backup code                 |

### Users

| Method | Path                                | Permission     | Purpose                     |
| ------ | ----------------------------------- | -------------- | --------------------------- |
| GET    | `/api/users`                        | `manage_users` | List, search and filter     |
| GET    | `/api/users/meta`                   | `manage_users` | Assignable roles and settings |
| GET    | `/api/users/{id}`                   | `manage_users` | Load a user for editing     |
| POST   | `/api/users`                        | `manage_users` | Create a user               |
| PATCH  | `/api/users/{id}`                   | `manage_users` | Update a user               |
| DELETE | `/api/users/{id}`                   | `manage_users` | Delete a user               |
| POST   | `/api/users/{id}/verify`            | `manage_users` | Approve a pending user      |
| PUT    | `/api/users/registration-approval`  | `manage_users` | Toggle the approval gate    |

### Roles and settings

| Method | Path                          | Permission      | Purpose                    |
| ------ | ----------------------------- | --------------- | -------------------------- |
| GET    | `/api/roles`                  | `manage_roles`  | Roles and permission registry |
| POST   | `/api/roles`                  | `manage_roles`  | Create a role              |
| PATCH  | `/api/roles/{id}`             | `manage_roles`  | Update a role              |
| DELETE | `/api/roles/{id}`             | `manage_roles`  | Delete an unused role      |
| PATCH  | `/api/settings/profile`       | session         | Update the current profile |
| DELETE | `/api/settings/account`       | session + confirm | Delete the current account |
| GET    | `/api/dashboard`              | `view_admin`    | Dashboard aggregate        |
| GET    | `/api/health`                 | public          | Database health check      |

## Middleware

- `rate_limit_auth` limits mutating auth requests per IP.
- `same_origin` rejects state-changing requests from an origin outside `APP_URL` and `ALLOWED_ORIGINS`.
- `security_headers` adds `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy` and `Permissions-Policy`.
- The session cookie is HTTP-only, SameSite=Lax and secure when `COOKIE_SECURE=true`.

## Frontend guards

- `RequireAuth` redirects guests to `/login?next=<path>`.
- `RequirePermission` redirects to `/403`.
- The server re-checks every permission. The guards are a UX layer, not the security boundary.
