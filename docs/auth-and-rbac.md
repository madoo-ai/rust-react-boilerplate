# Auth and RBAC

## Authentication

The auth use case lives in `api/src/app/usecase/auth/`. It depends on ports, not on SQLx or SMTP.

- Email and password are enabled.
- Email verification is sent on sign-up but is not required to sign in.
- Password reset uses a one-hour token and revokes every session for the user.
- Two-factor authentication uses TOTP and ten single-use backup codes.
- Sessions are stored in PostgreSQL. The cookie holds a random token. The database stores only its SHA-256 hash.
- Each request loads the session, user and permissions from PostgreSQL. There is no cookie cache, so a suspension or role change applies immediately.
- Trusted devices last 30 days.
- Auth endpoints are rate limited per IP in the HTTP middleware.
- Sign-up does not create a session. The client inspects the new account status and then signs in.

## Registration and approval

1. The public registration form calls `POST /api/auth/sign-up/email`.
2. The use case reads `settings.registration_approval` and sets `status` to `pending` or `active`. It assigns the default `user` role.
3. The form reads `result.user.status`:
   - `pending` -> `/login?error=account_pending`
   - `active` -> explicit sign-in, then `/dashboard`
4. The sign-in use case rejects pending and suspended users with `ACCOUNT_PENDING` or `ACCOUNT_SUSPENDED`.
5. HR verifies a pending account from the Users page. The account becomes `active` and can sign in.

The approval switch is stored in `settings` and is gated by `manage_users`.

## Password reset and email verification

- The mailer renders the two templates in `api/src/adapter/smtp/mod.rs`.
- In development, Mailpit receives mail at `http://localhost:8025`.
- If SMTP is not configured or the send fails, the adapter logs the link and the auth request still succeeds.
- Reset and verification links open the React routes `/reset-password` and `/verify-email`. Those pages call the API.

## Two-factor authentication

- TOTP is six digits with a 30-second period and one period of clock skew.
- The Security settings page asks for the password, returns the TOTP URI and backup codes, shows a QR code and verifies the first code before the account is marked as enabled.
- Disabling 2FA also requires the password.
- A successful password sign-in with 2FA enabled returns `twoFactorRedirect: true` and sets a short-lived challenge cookie.
- A failed code increments a counter. Five failures lock the challenge for 15 minutes.
- A used backup code is removed. The remaining codes stay valid.

## Confirm password

Sensitive actions can require a fresh password confirmation:

- `/confirm-password` calls `POST /api/auth/confirm-password`.
- On success the API sets a signed, HTTP-only `pwd_confirmed` cookie for 15 minutes.
- Account deletion checks the cookie.
- The cookie is signed with `APP_SECRET`, so a client cannot forge a timestamp.

## RBAC model

```text
users.role_id -> roles -< role_permissions >- permissions
```

A user has one role. Roles and permissions are stored in PostgreSQL and seeded from code:

- `api/src/entity/permission.rs` - permission keys, labels and helpers.
- `api/src/entity/role.rs` - role names, labels, descriptions and the default permission set.
- `api/src/entity/authorization.rs` - `can_manage_user`, `can_delete_user`, `can_assign_role`, `can_manage_role`.

### Seeded roles

| Role         | Permissions                                  |
| ------------ | -------------------------------------------- |
| `superadmin` | `view_admin`, `manage_users`, `manage_roles` |
| `hr`         | `view_admin`, `manage_users`                 |
| `user`       | none                                         |

### Enforcement layers

1. **Session guard** - `CurrentSession` in every protected handler.
2. **Use case guard** - `Actor::require(Permission::ManageUsers)` or similar.
3. **Policy** - the fine rules:
   - HR cannot manage a superadmin.
   - Only `manage_roles` can assign `superadmin`.
   - Nobody can delete themselves from the admin UI.
   - The last superadmin cannot be deleted or demoted.
   - The `superadmin` role is immutable.
   - The default `user` role cannot be renamed or deleted.
4. **Navigation** - `filterNavItems` hides links the user cannot use. This is a UX layer, not the security boundary.

## Adding a role or permission

1. Add the permission key and label to `api/src/entity/permission.rs`.
2. Add the role or adjust the permission set in `api/src/entity/role.rs`.
3. Run `make db-seed`.
4. Add the key to both message catalogues under `permissions.*`.
5. Gate the handler with `CurrentSession` and the use case with `Actor::require`.
6. Add a unit test in `api/src/entity/authorization.rs` and a web navigation test.
