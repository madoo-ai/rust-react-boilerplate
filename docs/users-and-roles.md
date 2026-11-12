# Users and roles

## Users

The Users page is at `/admin/users` and requires `manage_users`.

### List

- Search matches name or email, case-insensitive.
- Filter by role and status.
- The API paginates with ten rows per page.
- The registration approval switch sits above the table.
- Each row shows name/email, role, status, creation date and actions.
- Pending users have a Verify action. Active and suspended users do not.

### Create

`/admin/users/create` renders the shared user form. Fields:

- name, email, phone, job title, department
- role (only roles the actor may assign)
- status (`active`, `pending`, `suspended`)
- password (required, 8-128 characters)

The use case hashes the password, then the SQLx adapter inserts the user and the credential account in one transaction.

### Edit

`/admin/users/:id/edit` reuses the form. The password field is optional. An empty value keeps the current password. Changing the email marks it unverified and sends a verification link. Changing the status away from `active` revokes every session for the user.

### Delete

Deletion is confirmed with an `AlertDialog`. The use case refuses to delete:

- yourself,
- another superadmin when you are not a superadmin,
- the last superadmin.

Deleting a user cascades sessions, accounts and 2FA records.

### Protection rules

| Rule                                                          | Where                                              |
| ------------------------------------------------------------- | -------------------------------------------------- |
| HR can manage users but not roles                             | `can_assign_role`, `Actor::require`                |
| HR cannot edit or delete a superadmin                         | `can_manage_user`                                  |
| Only `manage_roles` can assign `superadmin`                   | `can_assign_role`                                  |
| You cannot delete yourself                                    | `can_delete_user`                                  |
| The last superadmin cannot be deleted                         | `can_delete_user` plus the superadmin count        |
| You cannot change your own role or status from the admin form | `UserService::update`                              |

## Roles

The Roles page is at `/admin/roles` and requires `manage_roles`.

### List

- Name, label, description, assigned permissions and user count.
- The `superadmin` row shows a Protected badge and has no actions.
- Edit opens a dialog with the label, description and permission checkboxes.
- Delete is disabled when the role is `superadmin`, the default `user` role, or assigned to at least one user.

### Create

The create dialog validates:

- name: lowercase letters, numbers and underscores, 2-60 characters
- label: required, up to 80 characters
- description: optional, up to 240 characters
- permissions: any subset of the permission registry

Role names are unique. Permission rows replace the previous set in a transaction.

### Edit

- The superadmin role cannot be edited.
- The default `user` role can change its label, description and permissions, but not its name.
- Renaming another role is allowed when the new name is unique.

### Delete

- The superadmin role is immutable.
- The default `user` role cannot be deleted.
- A role assigned to users cannot be deleted.
- A custom role with no users can be deleted. Its permission rows cascade.

## Adding a user or role programmatically

Call `UserService` or `RoleService` from a new use case. Reuse the entity policies. Do not write a second authorization path.
