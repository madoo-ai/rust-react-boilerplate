"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation } from "@tanstack/react-query";
import { Mail01Icon as Mail } from "@x1nx3r/altics-react-icons";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { Link } from "react-router-dom";
import { z } from "zod";

import { FormAlert } from "@/components/auth/form-alert";
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
import { errorMessage } from "@/lib/errors";
import { translateFieldError, useTranslations } from "@/lib/i18n";

const schema = z.object({
  email: z.email("validation.email"),
});

type ForgotValues = z.infer<typeof schema>;

export function ForgotPasswordForm() {
  const t = useTranslations("auth.forgot");
  const tFields = useTranslations("auth.fields");
  const tMessages = useTranslations("auth.messages");
  const tAll = useTranslations();
  const tValidation = useTranslations("validation");
  const [sent, setSent] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);

  const form = useForm<ForgotValues>({
    resolver: zodResolver(schema),
    defaultValues: { email: "" },
  });

  const fieldError = (message?: string) =>
    translateFieldError(message, tValidation);

  const mutation = useMutation({
    mutationFn: (values: ForgotValues) => authApi.forgotPassword(values.email),
    onSuccess: () => setSent(true),
    onError: (error) => setFormError(errorMessage(error, tAll)),
  });

  function onSubmit(values: ForgotValues) {
    setFormError(null);
    mutation.mutate(values);
  }

  if (sent) {
    return (
      <div className="flex flex-col gap-4">
        <FormAlert message={tMessages("emailSent")} variant="default" />
        <Button asChild variant="outline" className="w-full">
          <Link to="/login">{t("back")}</Link>
        </Button>
      </div>
    );
  }

  return (
    <form onSubmit={form.handleSubmit(onSubmit)} noValidate>
      <FieldGroup className="gap-4">
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
        <Button type="submit" className="w-full" disabled={mutation.isPending}>
          {mutation.isPending ? (
            <Spinner data-icon="inline-start" />
          ) : (
            <Mail data-icon="inline-start" />
          )}
          {t("submit")}
        </Button>
      </FieldGroup>
    </form>
  );
}
