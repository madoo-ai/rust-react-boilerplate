"use client";

import { useMutation } from "@tanstack/react-query";
import {
  Key01Icon as KeyRound,
  ShieldTickIcon as ShieldCheck,
} from "@x1nx3r/altics-react-icons";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { REGEXP_ONLY_DIGITS } from "input-otp";

import { FormAlert } from "@/components/auth/form-alert";
import { Button } from "@/components/ui/button";
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import {
  InputOTP,
  InputOTPGroup,
  InputOTPSlot,
} from "@/components/ui/input-otp";
import { Spinner } from "@/components/ui/spinner";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { authApi } from "@/lib/auth-api";
import { useAuth } from "@/lib/auth";
import { errorMessage } from "@/lib/errors";
import { useTranslations } from "@/lib/i18n";

export function TwoFactorForm() {
  const t = useTranslations("auth.twoFactor");
  const tFields = useTranslations("auth.fields");
  const tAll = useTranslations();
  const navigate = useNavigate();
  const { refresh } = useAuth();
  const [code, setCode] = useState("");
  const [backupCode, setBackupCode] = useState("");
  const [error, setError] = useState<string | null>(null);

  async function complete() {
    await refresh();
    navigate("/dashboard", { replace: true });
  }

  const totp = useMutation({
    mutationFn: () => authApi.twoFactor.verifyTotp(code, true),
    onSuccess: complete,
    onError: (mutationError) => setError(errorMessage(mutationError, tAll)),
  });

  const backup = useMutation({
    mutationFn: () => authApi.twoFactor.verifyBackupCode(backupCode, true),
    onSuccess: complete,
    onError: (mutationError) => setError(errorMessage(mutationError, tAll)),
  });

  return (
    <div className="flex flex-col gap-4">
      <FormAlert message={error} />
      <Tabs defaultValue="totp">
        <TabsList className="grid w-full grid-cols-2">
          <TabsTrigger value="totp">{t("totpTab")}</TabsTrigger>
          <TabsTrigger value="backup">{t("backupTab")}</TabsTrigger>
        </TabsList>
        <TabsContent value="totp">
          <form
            className="pt-4"
            onSubmit={(event) => {
              event.preventDefault();
              setError(null);
              totp.mutate();
            }}
          >
            <FieldGroup className="gap-4">
              <Field>
                <FieldLabel htmlFor="totp-code">{tFields("code")}</FieldLabel>
                <InputOTP
                  id="totp-code"
                  maxLength={6}
                  pattern={REGEXP_ONLY_DIGITS}
                  value={code}
                  onChange={setCode}
                  autoFocus
                >
                  <InputOTPGroup>
                    {Array.from({ length: 6 }).map((_, index) => (
                      <InputOTPSlot key={index} index={index} />
                    ))}
                  </InputOTPGroup>
                </InputOTP>
              </Field>
              <Button
                type="submit"
                className="w-full"
                disabled={code.length !== 6 || totp.isPending}
              >
                {totp.isPending ? (
                  <Spinner data-icon="inline-start" />
                ) : (
                  <ShieldCheck data-icon="inline-start" />
                )}
                {t("submit")}
              </Button>
            </FieldGroup>
          </form>
        </TabsContent>
        <TabsContent value="backup">
          <form
            className="pt-4"
            onSubmit={(event) => {
              event.preventDefault();
              setError(null);
              backup.mutate();
            }}
          >
            <FieldGroup className="gap-4">
              <Field>
                <FieldLabel htmlFor="backup-code">
                  {t("backupTitle")}
                </FieldLabel>
                <Input
                  id="backup-code"
                  autoComplete="one-time-code"
                  value={backupCode}
                  onChange={(event) => setBackupCode(event.target.value)}
                  autoFocus
                />
                <FieldDescription>{t("backupDescription")}</FieldDescription>
              </Field>
              <Button
                type="submit"
                className="w-full"
                disabled={backupCode.length === 0 || backup.isPending}
              >
                {backup.isPending ? (
                  <Spinner data-icon="inline-start" />
                ) : (
                  <KeyRound data-icon="inline-start" />
                )}
                {t("backupSubmit")}
              </Button>
            </FieldGroup>
          </form>
        </TabsContent>
      </Tabs>
    </div>
  );
}
