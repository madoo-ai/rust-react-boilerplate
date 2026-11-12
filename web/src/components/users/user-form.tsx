"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Save01Icon as Save } from "@x1nx3r/altics-react-icons";
import { useState } from "react";
import { useForm, useWatch } from "react-hook-form";
import { Link, useNavigate } from "react-router-dom";
import { toast } from "sonner";
import { z } from "zod";

import { FormAlert } from "@/components/auth/form-alert";
import { PasswordInput } from "@/components/auth/password-input";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Field,
  FieldDescription,
  FieldError,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Spinner } from "@/components/ui/spinner";
import { ApiError, api } from "@/lib/api";
import { errorMessage } from "@/lib/errors";
import { translateFieldError, useTranslations } from "@/lib/i18n";
import type { RoleOption, UserDetail, UserStatus } from "@/types";

const baseSchema = z.object({
  name: z
    .string()
    .trim()
    .min(2, "validation.nameLength")
    .max(120, "validation.nameLength"),
  email: z.email("validation.email"),
  phone: z.string().trim().max(40).optional(),
  jobTitle: z.string().trim().max(120).optional(),
  department: z.string().trim().max(120).optional(),
  role: z.string().min(1, "validation.invalidRole"),
  status: z.enum(["pending", "active", "suspended"]),
  password: z.string().max(128).optional(),
});

type UserFormValues = z.infer<typeof baseSchema>;

export function UserForm({
  mode,
  roles,
  user,
}: {
  mode: "create" | "edit";
  roles: RoleOption[];
  user?: UserDetail;
}) {
  const t = useTranslations("users");
  const tForm = useTranslations("users.form");
  const tCommon = useTranslations("common");
  const tValidation = useTranslations("validation");
  const tActions = useTranslations("users.actions");
  const tAll = useTranslations();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [formError, setFormError] = useState<string | null>(null);

  const schema = baseSchema.superRefine((values, ctx) => {
    const password = values.password ?? "";

    if (mode === "create" && password.length < 8) {
      ctx.addIssue({
        code: "custom",
        path: ["password"],
        message: "validation.passwordLength",
      });
    }

    if (mode === "edit" && password.length > 0 && password.length < 8) {
      ctx.addIssue({
        code: "custom",
        path: ["password"],
        message: "validation.passwordLength",
      });
    }
  });

  const defaultRole =
    roles.find((role) => role.name === "user")?.name ?? roles[0]?.name ?? "";

  const form = useForm<UserFormValues>({
    resolver: zodResolver(schema),
    defaultValues: {
      name: user?.name ?? "",
      email: user?.email ?? "",
      phone: user?.phoneNumber ?? "",
      jobTitle: user?.jobTitle ?? "",
      department: user?.department ?? "",
      role: user?.role ?? defaultRole,
      status: user?.status ?? "active",
      password: "",
    },
  });

  const fieldError = (message?: string) =>
    translateFieldError(message, tValidation);

  const mutation = useMutation({
    mutationFn: (values: UserFormValues) => {
      const payload = { ...values, phone: values.phone ?? "" };

      if (mode === "create") {
        return api.post("/users", payload);
      }

      if (!user) throw new Error(tAll("errors.userNotFound"));
      return api.patch(`/users/${user.id}`, payload);
    },
    onSuccess: () => {
      toast.success(
        t(mode === "create" ? "messages.created" : "messages.updated"),
      );
      void queryClient.invalidateQueries({ queryKey: ["users"] });
      void queryClient.invalidateQueries({ queryKey: ["dashboard"] });
      navigate("/admin/users");
    },
    onError: (error) => {
      if (error instanceof ApiError && error.fields) {
        for (const [field, message] of Object.entries(error.fields)) {
          form.setError(field as keyof UserFormValues, { message });
        }
      }
      setFormError(errorMessage(error, tAll));
    },
  });

  function onSubmit(values: UserFormValues) {
    setFormError(null);
    mutation.mutate(values);
  }

  const role = useWatch({ control: form.control, name: "role" });
  const status = useWatch({ control: form.control, name: "status" });

  return (
    <form onSubmit={form.handleSubmit(onSubmit)} noValidate>
      <Card>
        <CardHeader>
          <CardTitle>{tForm("title")}</CardTitle>
          <CardDescription>{tForm("description")}</CardDescription>
        </CardHeader>
        <CardContent>
          <FieldGroup className="grid gap-4 md:grid-cols-2">
            <div className="md:col-span-2">
              <FormAlert message={formError} />
            </div>
            <Field
              className="md:col-span-2"
              data-invalid={!!form.formState.errors.name}
            >
              <FieldLabel htmlFor="name">{tForm("name")}</FieldLabel>
              <Input
                id="name"
                autoComplete="name"
                aria-invalid={!!form.formState.errors.name}
                {...form.register("name")}
              />
              <FieldError>
                {fieldError(form.formState.errors.name?.message)}
              </FieldError>
            </Field>
            <Field data-invalid={!!form.formState.errors.email}>
              <FieldLabel htmlFor="email">{tForm("email")}</FieldLabel>
              <Input
                id="email"
                type="email"
                autoComplete="email"
                aria-invalid={!!form.formState.errors.email}
                {...form.register("email")}
              />
              <FieldError>
                {fieldError(form.formState.errors.email?.message)}
              </FieldError>
            </Field>
            <Field>
              <FieldLabel htmlFor="phone">{tForm("phone")}</FieldLabel>
              <Input
                id="phone"
                autoComplete="tel"
                {...form.register("phone")}
              />
            </Field>
            <Field>
              <FieldLabel htmlFor="jobTitle">{tForm("jobTitle")}</FieldLabel>
              <Input id="jobTitle" {...form.register("jobTitle")} />
            </Field>
            <Field>
              <FieldLabel htmlFor="department">
                {tForm("department")}
              </FieldLabel>
              <Input id="department" {...form.register("department")} />
            </Field>
            <Field data-invalid={!!form.formState.errors.role}>
              <FieldLabel htmlFor="role">{tForm("role")}</FieldLabel>
              <Select
                value={role}
                onValueChange={(value) =>
                  form.setValue("role", value, { shouldValidate: true })
                }
              >
                <SelectTrigger id="role" aria-label={tForm("role")}>
                  <SelectValue placeholder={tForm("role")} />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    {roles.map((option) => (
                      <SelectItem key={option.name} value={option.name}>
                        {option.label}
                      </SelectItem>
                    ))}
                  </SelectGroup>
                </SelectContent>
              </Select>
              <FieldError>
                {fieldError(form.formState.errors.role?.message)}
              </FieldError>
            </Field>
            <Field data-invalid={!!form.formState.errors.status}>
              <FieldLabel htmlFor="status">{tForm("status")}</FieldLabel>
              <Select
                value={status}
                onValueChange={(value) =>
                  form.setValue("status", value as UserStatus, {
                    shouldValidate: true,
                  })
                }
              >
                <SelectTrigger id="status" aria-label={tForm("status")}>
                  <SelectValue placeholder={tForm("status")} />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    <SelectItem value="active">{tCommon("active")}</SelectItem>
                    <SelectItem value="pending">
                      {tCommon("pending")}
                    </SelectItem>
                    <SelectItem value="suspended">
                      {tCommon("suspended")}
                    </SelectItem>
                  </SelectGroup>
                </SelectContent>
              </Select>
              <FieldError>
                {fieldError(form.formState.errors.status?.message)}
              </FieldError>
            </Field>
            <Field
              className="md:col-span-2"
              data-invalid={!!form.formState.errors.password}
            >
              <FieldLabel htmlFor="password">{tForm("password")}</FieldLabel>
              <PasswordInput
                id="password"
                autoComplete="new-password"
                aria-invalid={!!form.formState.errors.password}
                {...form.register("password")}
              />
              {mode === "edit" ? (
                <FieldDescription>{tForm("passwordHint")}</FieldDescription>
              ) : (
                <FieldDescription>
                  {tForm("passwordHintCreate")}
                </FieldDescription>
              )}
              <FieldError>
                {fieldError(form.formState.errors.password?.message)}
              </FieldError>
            </Field>
          </FieldGroup>
        </CardContent>
        <CardFooter className="justify-end gap-2 border-t">
          <Button asChild variant="outline">
            <Link to="/admin/users">{tCommon("cancel")}</Link>
          </Button>
          <Button type="submit" disabled={mutation.isPending}>
            {mutation.isPending ? (
              <Spinner data-icon="inline-start" />
            ) : (
              <Save data-icon="inline-start" />
            )}
            {mode === "create" ? tActions("create") : tActions("save")}
          </Button>
        </CardFooter>
      </Card>
    </form>
  );
}
