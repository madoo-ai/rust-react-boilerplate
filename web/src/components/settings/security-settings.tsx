"use client";

import { useMutation } from "@tanstack/react-query";
import {
  Key01Icon as KeyRound,
  ShieldOffIcon as ShieldOff,
  ShieldTickIcon as ShieldCheck,
} from "@x1nx3r/altics-react-icons";
import { REGEXP_ONLY_DIGITS } from "input-otp";
import { useState } from "react";
import QRCode from "react-qr-code";
import { toast } from "sonner";

import { FormAlert } from "@/components/auth/form-alert";
import { PasswordInput } from "@/components/auth/password-input";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Field, FieldDescription, FieldLabel } from "@/components/ui/field";
import {
  InputOTP,
  InputOTPGroup,
  InputOTPSlot,
} from "@/components/ui/input-otp";
import { Spinner } from "@/components/ui/spinner";
import { authApi } from "@/lib/auth-api";
import { useAuth } from "@/lib/auth";
import { errorMessage } from "@/lib/errors";
import { useTranslations } from "@/lib/i18n";

export function SecuritySettings({
  twoFactorEnabled,
}: {
  twoFactorEnabled: boolean;
}) {
  const t = useTranslations("settings.security");
  const tCommon = useTranslations("common");
  const tAll = useTranslations();
  const { refresh } = useAuth();
  const [enableOpen, setEnableOpen] = useState(false);
  const [disableOpen, setDisableOpen] = useState(false);
  const [step, setStep] = useState<"password" | "verify">("password");
  const [password, setPassword] = useState("");
  const [code, setCode] = useState("");
  const [totpURI, setTotpURI] = useState("");
  const [secret, setSecret] = useState("");
  const [backupCodes, setBackupCodes] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);

  function resetEnable() {
    setStep("password");
    setPassword("");
    setCode("");
    setTotpURI("");
    setSecret("");
    setBackupCodes([]);
    setError(null);
  }

  function handleEnableOpenChange(open: boolean) {
    setEnableOpen(open);
    if (!open) resetEnable();
  }

  const enable = useMutation({
    mutationFn: () => authApi.twoFactor.enable(password),
    onSuccess: (setup) => {
      setTotpURI(setup.totpURI);
      setSecret(setup.secret);
      setBackupCodes(setup.backupCodes);
      setStep("verify");
    },
    onError: (mutationError) => setError(errorMessage(mutationError, tAll)),
  });

  const verify = useMutation({
    mutationFn: () => authApi.twoFactor.verify(code),
    onSuccess: async () => {
      toast.success(t("enabledMessage"));
      setEnableOpen(false);
      resetEnable();
      await refresh();
    },
    onError: (mutationError) => setError(errorMessage(mutationError, tAll)),
  });

  const disable = useMutation({
    mutationFn: () => authApi.twoFactor.disable(password),
    onSuccess: async () => {
      toast.success(t("disabledMessage"));
      setDisableOpen(false);
      setPassword("");
      await refresh();
    },
    onError: (mutationError) => setError(errorMessage(mutationError, tAll)),
  });

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
          <div className="flex items-center gap-3">
            {twoFactorEnabled ? (
              <ShieldCheck className="text-primary" />
            ) : (
              <ShieldOff className="text-muted-foreground" />
            )}
            <div className="flex flex-col gap-1">
              <span className="font-medium">{t("twoFactor")}</span>
              <Badge
                variant={twoFactorEnabled ? "default" : "secondary"}
                className="w-fit"
              >
                {twoFactorEnabled ? t("enabled") : t("disabled")}
              </Badge>
            </div>
          </div>
          {twoFactorEnabled ? (
            <Button
              variant="outline"
              onClick={() => {
                setError(null);
                setPassword("");
                setDisableOpen(true);
              }}
            >
              <ShieldOff data-icon="inline-start" />
              {t("disable")}
            </Button>
          ) : (
            <Button
              onClick={() => {
                setError(null);
                setEnableOpen(true);
              }}
            >
              <KeyRound data-icon="inline-start" />
              {t("enable")}
            </Button>
          )}
        </div>
      </CardContent>

      <Dialog open={enableOpen} onOpenChange={handleEnableOpenChange}>
        <DialogContent className="sm:max-w-lg">
          <DialogHeader>
            <DialogTitle>{t("enable")}</DialogTitle>
            <DialogDescription>{t("description")}</DialogDescription>
          </DialogHeader>
          {step === "password" ? (
            <form
              className="flex flex-col gap-4"
              onSubmit={(event) => {
                event.preventDefault();
                setError(null);
                enable.mutate();
              }}
            >
              <FormAlert message={error} />
              <Field>
                <FieldLabel htmlFor="enable-password">
                  {t("password")}
                </FieldLabel>
                <PasswordInput
                  id="enable-password"
                  autoComplete="current-password"
                  value={password}
                  onChange={(event) => setPassword(event.target.value)}
                />
              </Field>
              <DialogFooter>
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => handleEnableOpenChange(false)}
                  disabled={enable.isPending}
                >
                  {tCommon("cancel")}
                </Button>
                <Button type="submit" disabled={!password || enable.isPending}>
                  {enable.isPending ? (
                    <Spinner data-icon="inline-start" />
                  ) : null}
                  {t("enable")}
                </Button>
              </DialogFooter>
            </form>
          ) : (
            <form
              className="flex flex-col gap-4"
              onSubmit={(event) => {
                event.preventDefault();
                setError(null);
                verify.mutate();
              }}
            >
              <FormAlert message={error} />
              <div className="flex flex-col items-center gap-3 rounded-lg border bg-white p-4">
                {totpURI ? <QRCode value={totpURI} size={168} /> : null}
              </div>
              <p className="text-sm text-muted-foreground">{t("scan")}</p>
              <p className="text-xs text-muted-foreground">
                {t("secret")}: <code className="font-mono">{secret}</code>
              </p>
              <div className="rounded-lg bg-muted p-3">
                <p className="mb-2 text-sm font-medium">{t("backupCodes")}</p>
                <div className="grid grid-cols-2 gap-1 font-mono text-xs">
                  {backupCodes.map((backupCode) => (
                    <span key={backupCode}>{backupCode}</span>
                  ))}
                </div>
              </div>
              <Field>
                <FieldLabel htmlFor="totp-code">{t("code")}</FieldLabel>
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
                <FieldDescription>{t("verify")}</FieldDescription>
              </Field>
              <DialogFooter>
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => handleEnableOpenChange(false)}
                  disabled={verify.isPending}
                >
                  {tCommon("cancel")}
                </Button>
                <Button
                  type="submit"
                  disabled={code.length !== 6 || verify.isPending}
                >
                  {verify.isPending ? (
                    <Spinner data-icon="inline-start" />
                  ) : null}
                  {t("verify")}
                </Button>
              </DialogFooter>
            </form>
          )}
        </DialogContent>
      </Dialog>

      <Dialog open={disableOpen} onOpenChange={setDisableOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>{t("disable")}</DialogTitle>
            <DialogDescription>{t("description")}</DialogDescription>
          </DialogHeader>
          <form
            className="flex flex-col gap-4"
            onSubmit={(event) => {
              event.preventDefault();
              setError(null);
              disable.mutate();
            }}
          >
            <FormAlert message={error} />
            <Field>
              <FieldLabel htmlFor="disable-password">
                {t("password")}
              </FieldLabel>
              <PasswordInput
                id="disable-password"
                autoComplete="current-password"
                value={password}
                onChange={(event) => setPassword(event.target.value)}
              />
            </Field>
            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => setDisableOpen(false)}
                disabled={disable.isPending}
              >
                {tCommon("cancel")}
              </Button>
              <Button type="submit" disabled={!password || disable.isPending}>
                {disable.isPending ? (
                  <Spinner data-icon="inline-start" />
                ) : null}
                {t("disable")}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </Card>
  );
}
