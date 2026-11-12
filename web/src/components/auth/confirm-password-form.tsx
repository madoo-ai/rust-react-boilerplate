"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation } from "@tanstack/react-query";
import { ShieldTickIcon as ShieldCheck } from "@x1nx3r/altics-react-icons";
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
import { Spinner } from "@/components/ui/spinner";
import { authApi } from "@/lib/auth-api";
import { useAuth } from "@/lib/auth";
import { errorMessage } from "@/lib/errors";
import { translateFieldError, useTranslations } from "@/lib/i18n";

const schema = z.object({
  password: z.string().min(1, "validation.required"),
});

type ConfirmValues = z.infer<typeof schema>;

function safeNextPath(next: string | undefined) {
  if (!next) return "/settings/profile";
  if (!next.startsWith("/") || next.startsWith("//"))
    return "/settings/profile";
  return next;
}

export function ConfirmPasswordForm({ next }: { next?: string }) {
  const t = useTranslations("auth.confirmPassword");
  const tFields = useTranslations("auth.fields");
  const tAll = useTranslations();
  const tValidation = useTranslations("validation");
  const navigate = useNavigate();
  const { refresh } = useAuth();
  const [formError, setFormError] = useState<string | null>(null);

  const form = useForm<ConfirmValues>({
    resolver: zodResolver(schema),
    defaultValues: { password: "" },
  });

  const fieldError = (message?: string) =>
    translateFieldError(message, tValidation);

  const mutation = useMutation({
    mutationFn: (values: ConfirmValues) =>
      authApi.confirmPassword(values.password),
    onSuccess: async () => {
      await refresh();
      navigate(safeNextPath(next), { replace: true });
    },
    onError: (error) => setFormError(errorMessage(error, tAll)),
  });

  function onSubmit(values: ConfirmValues) {
    setFormError(null);
    mutation.mutate(values);
  }

  return (
    <form onSubmit={form.handleSubmit(onSubmit)} noValidate>
      <FieldGroup className="gap-4">
        <FormAlert message={formError} />
        <Field data-invalid={!!form.formState.errors.password}>
          <FieldLabel htmlFor="password">{tFields("password")}</FieldLabel>
          <PasswordInput
            id="password"
            autoComplete="current-password"
            autoFocus
            aria-invalid={!!form.formState.errors.password}
            {...form.register("password")}
          />
          <FieldError>
            {fieldError(form.formState.errors.password?.message)}
          </FieldError>
        </Field>
        <Button type="submit" className="w-full" disabled={mutation.isPending}>
          {mutation.isPending ? (
            <Spinner data-icon="inline-start" />
          ) : (
            <ShieldCheck data-icon="inline-start" />
          )}
          {t("submit")}
        </Button>
      </FieldGroup>
    </form>
  );
}
