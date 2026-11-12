"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  Pencil01Icon as Pencil,
  PlusIcon as Plus,
  Save01Icon as Save,
} from "@x1nx3r/altics-react-icons";
import { useState } from "react";
import { useForm, useWatch } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";

import { FormAlert } from "@/components/auth/form-alert";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  Field,
  FieldDescription,
  FieldError,
  FieldGroup,
  FieldLabel,
  FieldLegend,
  FieldSet,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Spinner } from "@/components/ui/spinner";
import { Textarea } from "@/components/ui/textarea";
import { ApiError, api } from "@/lib/api";
import { errorMessage } from "@/lib/errors";
import { translateFieldError, useTranslations } from "@/lib/i18n";
import type { PermissionOption } from "@/types";

const roleSchema = z.object({
  name: z
    .string()
    .trim()
    .min(2, "validation.required")
    .max(60, "validation.max")
    .regex(/^[a-z0-9_]+$/, "validation.roleName"),
  label: z
    .string()
    .trim()
    .min(1, "validation.required")
    .max(80, "validation.max"),
  description: z.string().trim().max(240).optional().or(z.literal("")),
  permissions: z.array(z.string()),
});

type RoleInput = z.infer<typeof roleSchema>;

type EditableRole = {
  id: string;
  name: string;
  label: string;
  description: string | null;
  permissions: string[];
};

function RoleDialog({
  mode,
  role,
  permissions,
  trigger,
}: {
  mode: "create" | "edit";
  role?: EditableRole;
  permissions: PermissionOption[];
  trigger: React.ReactNode;
}) {
  const t = useTranslations("roles");
  const tForm = useTranslations("roles.form");
  const tActions = useTranslations("roles.actions");
  const tCommon = useTranslations("common");
  const tValidation = useTranslations("validation");
  const tPermissionNames = useTranslations("permissions");
  const tAll = useTranslations();
  const queryClient = useQueryClient();
  const [open, setOpen] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);

  const form = useForm<RoleInput>({
    resolver: zodResolver(roleSchema),
    defaultValues: {
      name: role?.name ?? "",
      label: role?.label ?? "",
      description: role?.description ?? "",
      permissions: role?.permissions ?? [],
    },
  });

  const fieldError = (message?: string) =>
    translateFieldError(message, tValidation);

  const mutation = useMutation({
    mutationFn: (values: RoleInput) =>
      mode === "create"
        ? api.post("/roles", values)
        : api.patch(`/roles/${role?.id}`, values),
    onSuccess: () => {
      setOpen(false);
      toast.success(
        t(mode === "create" ? "messages.created" : "messages.updated"),
      );
      void queryClient.invalidateQueries({ queryKey: ["roles"] });
      void queryClient.invalidateQueries({ queryKey: ["user-meta"] });
    },
    onError: (error) => {
      if (error instanceof ApiError && error.fields) {
        for (const [field, message] of Object.entries(error.fields)) {
          form.setError(field as keyof RoleInput, { message });
        }
      }
      setFormError(errorMessage(error, tAll));
    },
  });

  function onSubmit(values: RoleInput) {
    setFormError(null);
    mutation.mutate(values);
  }

  const selectedPermissions =
    useWatch({ control: form.control, name: "permissions" }) ?? [];
  const lockName = role?.name === "user";

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>{trigger}</DialogTrigger>
      <DialogContent className="sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>
            {mode === "create" ? t("create") : t("edit")}
          </DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <form
          className="flex flex-col gap-4"
          onSubmit={form.handleSubmit(onSubmit)}
          noValidate
        >
          <FormAlert message={formError} />
          <FieldGroup>
            <div className="grid gap-4 sm:grid-cols-2">
              <Field data-invalid={!!form.formState.errors.name}>
                <FieldLabel htmlFor="role-name">{tForm("name")}</FieldLabel>
                <Input
                  id="role-name"
                  disabled={lockName}
                  aria-invalid={!!form.formState.errors.name}
                  {...form.register("name")}
                />
                {lockName ? (
                  <FieldDescription>{tForm("nameHint")}</FieldDescription>
                ) : null}
                <FieldError>
                  {fieldError(form.formState.errors.name?.message)}
                </FieldError>
              </Field>
              <Field data-invalid={!!form.formState.errors.label}>
                <FieldLabel htmlFor="role-label">{tForm("label")}</FieldLabel>
                <Input
                  id="role-label"
                  aria-invalid={!!form.formState.errors.label}
                  {...form.register("label")}
                />
                <FieldError>
                  {fieldError(form.formState.errors.label?.message)}
                </FieldError>
              </Field>
            </div>
            <Field>
              <FieldLabel htmlFor="role-description">
                {tForm("description")}
              </FieldLabel>
              <Textarea
                id="role-description"
                rows={3}
                {...form.register("description")}
              />
            </Field>
            <FieldSet>
              <FieldLegend>{tForm("permissions")}</FieldLegend>
              <FieldGroup>
                {permissions.map((permission) => {
                  const checked = selectedPermissions.includes(permission.key);

                  return (
                    <Field key={permission.key} orientation="horizontal">
                      <Checkbox
                        id={`permission-${permission.key}`}
                        checked={checked}
                        onCheckedChange={(value) => {
                          const next = value
                            ? [...selectedPermissions, permission.key]
                            : selectedPermissions.filter(
                                (key) => key !== permission.key,
                              );
                          form.setValue("permissions", next, {
                            shouldValidate: true,
                          });
                        }}
                      />
                      <FieldLabel
                        htmlFor={`permission-${permission.key}`}
                        className="font-normal"
                      >
                        {tPermissionNames(permission.key)}
                      </FieldLabel>
                    </Field>
                  );
                })}
              </FieldGroup>
            </FieldSet>
          </FieldGroup>
          <DialogFooter>
            <DialogClose asChild>
              <Button type="button" variant="outline">
                {tCommon("cancel")}
              </Button>
            </DialogClose>
            <Button type="submit" disabled={mutation.isPending}>
              {mutation.isPending ? (
                <Spinner data-icon="inline-start" />
              ) : (
                <Save data-icon="inline-start" />
              )}
              {mode === "create" ? tActions("create") : tActions("save")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

export function CreateRoleDialog({
  permissions,
}: {
  permissions: PermissionOption[];
}) {
  const t = useTranslations("roles");

  return (
    <RoleDialog
      mode="create"
      permissions={permissions}
      trigger={
        <Button>
          <Plus data-icon="inline-start" />
          {t("create")}
        </Button>
      }
    />
  );
}

export function EditRoleDialog({
  role,
  permissions,
}: {
  role: EditableRole;
  permissions: PermissionOption[];
}) {
  const tCommon = useTranslations("common");

  return (
    <RoleDialog
      mode="edit"
      role={role}
      permissions={permissions}
      trigger={
        <Button variant="ghost" size="icon-sm" aria-label={tCommon("edit")}>
          <Pencil />
        </Button>
      }
    />
  );
}
