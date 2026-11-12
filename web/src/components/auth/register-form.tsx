"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation } from "@tanstack/react-query";
import { UserPlus01Icon as UserPlus } from "@x1nx3r/altics-react-icons";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { useNavigate } from "react-router-dom";
import { z } from "zod";

import { FormAlert } from "@/components/auth/form-alert";
import { PasswordInput } from "@/components/auth/password-input";
import { Button } from "@/components/ui/button";
import {
  Field,
  FieldError,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Spinner } from "@/components/ui/spinner";
import { authApi } from "@/lib/auth-api";
import { useAuth } from "@/lib/auth";
import { authErrorMessage } from "@/lib/errors";
import { translateFieldError, useTranslations } from "@/lib/i18n";

const schema = z
  .object({
    name: z
      .string()
      .min(2, "validation.nameLength")
      .max(120, "validation.nameLength"),
    email: z.email("validation.email"),
    password: z
      .string()
      .min(8, "validation.passwordLength")
      .max(128, "validation.passwordLength"),
    confirmPassword: z.string().min(1, "validation.required"),
  })
  .refine((values) => values.password === values.confirmPassword, {
    path: ["confirmPassword"],
    message: "validation.passwordMismatch",
  });

type RegisterValues = z.infer<typeof schema>;

export function RegisterForm() {
  const t = useTranslations("auth.register");
  const tFields = useTranslations("auth.fields");
  const tAll = useTranslations();
  const tValidation = useTranslations("validation");
  const navigate = useNavigate();
  const { refresh } = useAuth();
  const [formError, setFormError] = useState<string | null>(null);

  const form = useForm<RegisterValues>({
    resolver: zodResolver(schema),
    defaultValues: {
      name: "",
      email: "",
      password: "",
      confirmPassword: "",
    },
  });

  const fieldError = (message?: string) =>
    translateFieldError(message, tValidation);

  const mutation = useMutation({
    mutationFn: async (values: RegisterValues) => {
      const signUp = await authApi.signUp({
        name: values.name,
        email: values.email,
        password: values.password,
      });

      if (signUp.user.status === "pending") {
        navigate("/login?error=account_pending");
        return null;
      }

      return authApi.signIn({
        email: values.email,
        password: values.password,
        rememberMe: true,
      });
    },
    onSuccess: async (result) => {
      if (!result) return;

      if ("twoFactorRedirect" in result) {
        navigate("/two-factor");
        return;
      }

      await refresh();
      navigate("/dashboard", { replace: true });
    },
    onError: (error) => setFormError(authErrorMessage(error, tAll)),
  });

  function onSubmit(values: RegisterValues) {
    setFormError(null);
    mutation.mutate(values);
  }

  return (
    <form onSubmit={form.handleSubmit(onSubmit)} noValidate>
      <FieldGroup className="gap-4">
        <FormAlert message={formError} />
        <Field data-invalid={!!form.formState.errors.name}>
          <FieldLabel htmlFor="name">{tFields("name")}</FieldLabel>
          <Input
            id="name"
            autoComplete="name"
            autoFocus
            aria-invalid={!!form.formState.errors.name}
            {...form.register("name")}
          />
          <FieldError>
            {fieldError(form.formState.errors.name?.message)}
          </FieldError>
        </Field>
        <Field data-invalid={!!form.formState.errors.email}>
          <FieldLabel htmlFor="email">{tFields("email")}</FieldLabel>
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
        <Field data-invalid={!!form.formState.errors.password}>
          <FieldLabel htmlFor="password">{tFields("password")}</FieldLabel>
          <PasswordInput
            id="password"
            autoComplete="new-password"
            aria-invalid={!!form.formState.errors.password}
            {...form.register("password")}
          />
          <FieldError>
            {fieldError(form.formState.errors.password?.message)}
          </FieldError>
        </Field>
        <Field data-invalid={!!form.formState.errors.confirmPassword}>
          <FieldLabel htmlFor="confirmPassword">
            {tFields("confirmPassword")}
          </FieldLabel>
          <PasswordInput
            id="confirmPassword"
            autoComplete="new-password"
            aria-invalid={!!form.formState.errors.confirmPassword}
            {...form.register("confirmPassword")}
          />
          <FieldError>
            {fieldError(form.formState.errors.confirmPassword?.message)}
          </FieldError>
        </Field>
        <Button type="submit" className="w-full" disabled={mutation.isPending}>
          {mutation.isPending ? (
            <Spinner data-icon="inline-start" />
          ) : (
            <UserPlus data-icon="inline-start" />
          )}
          {t("submit")}
        </Button>
      </FieldGroup>
    </form>
  );
}
