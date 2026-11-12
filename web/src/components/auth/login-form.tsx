"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation } from "@tanstack/react-query";
import { LogIn01Icon as LogIn } from "@x1nx3r/altics-react-icons";
import { useState } from "react";
import { useForm, useWatch } from "react-hook-form";
import { Link, useNavigate } from "react-router-dom";
import { z } from "zod";

import { FormAlert } from "@/components/auth/form-alert";
import { PasswordInput } from "@/components/auth/password-input";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
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

const schema = z.object({
  email: z.email("validation.email"),
  password: z.string().min(1, "validation.required"),
  rememberMe: z.boolean(),
});

type LoginValues = z.infer<typeof schema>;

function safeNextPath(next: string | undefined) {
  if (!next) return "/dashboard";
  if (!next.startsWith("/") || next.startsWith("//")) return "/dashboard";
  return next;
}

export function LoginForm({
  next,
  initialError,
  notice,
}: {
  next?: string;
  initialError?: string;
  notice?: string;
}) {
  const t = useTranslations("auth.login");
  const tFields = useTranslations("auth.fields");
  const tAll = useTranslations();
  const tValidation = useTranslations("validation");
  const navigate = useNavigate();
  const { refresh } = useAuth();
  const [formError, setFormError] = useState<string | null>(
    initialError === "account_inactive"
      ? tAll("errors.accountInactive")
      : initialError === "account_pending"
        ? tAll("auth.messages.pendingApproval")
        : null,
  );

  const form = useForm<LoginValues>({
    resolver: zodResolver(schema),
    defaultValues: { email: "", password: "", rememberMe: true },
  });

  const fieldError = (message?: string) =>
    translateFieldError(message, tValidation);

  const rememberMe = useWatch({ control: form.control, name: "rememberMe" });

  const mutation = useMutation({
    mutationFn: authApi.signIn,
    onSuccess: async (data) => {
      if ("twoFactorRedirect" in data) {
        navigate("/two-factor");
        return;
      }

      await refresh();
      navigate(safeNextPath(next), { replace: true });
    },
    onError: (error) => setFormError(authErrorMessage(error, tAll)),
  });

  function onSubmit(values: LoginValues) {
    setFormError(null);
    mutation.mutate(values);
  }

  return (
    <form onSubmit={form.handleSubmit(onSubmit)} noValidate>
      <FieldGroup className="gap-4">
        <FormAlert message={notice} variant="default" />
        <FormAlert message={formError} />
        <Field data-invalid={!!form.formState.errors.email}>
          <FieldLabel htmlFor="email">{tFields("email")}</FieldLabel>
          <Input
            id="email"
            type="email"
            autoComplete="email"
            autoFocus
            aria-invalid={!!form.formState.errors.email}
            {...form.register("email")}
          />
          <FieldError>
            {fieldError(form.formState.errors.email?.message)}
          </FieldError>
        </Field>
        <Field data-invalid={!!form.formState.errors.password}>
          <div className="flex items-center justify-between gap-2">
            <FieldLabel htmlFor="password">{tFields("password")}</FieldLabel>
            <Link
              to="/forgot-password"
              className="text-xs text-muted-foreground underline-offset-4 hover:text-primary hover:underline"
            >
              {t("forgot")}
            </Link>
          </div>
          <PasswordInput
            id="password"
            autoComplete="current-password"
            aria-invalid={!!form.formState.errors.password}
            {...form.register("password")}
          />
          <FieldError>
            {fieldError(form.formState.errors.password?.message)}
          </FieldError>
        </Field>
        <Field orientation="horizontal">
          <Checkbox
            id="rememberMe"
            checked={rememberMe}
            onCheckedChange={(checked) =>
              form.setValue("rememberMe", checked === true)
            }
          />
          <FieldLabel htmlFor="rememberMe" className="font-normal">
            {t("remember")}
          </FieldLabel>
        </Field>
        <Button type="submit" className="w-full" disabled={mutation.isPending}>
          {mutation.isPending ? (
            <Spinner data-icon="inline-start" />
          ) : (
            <LogIn data-icon="inline-start" />
          )}
          {t("submit")}
        </Button>
      </FieldGroup>
    </form>
  );
}
