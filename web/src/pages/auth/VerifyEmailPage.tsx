import { useMutation } from "@tanstack/react-query";
import { useEffect, useRef } from "react";
import { Link, useSearchParams } from "react-router-dom";

import { AuthCard } from "@/components/auth/auth-card";
import { FormAlert } from "@/components/auth/form-alert";
import { Button } from "@/components/ui/button";
import { authApi } from "@/lib/auth-api";
import { useTranslations } from "@/lib/i18n";

export function VerifyEmailPage() {
  const [params] = useSearchParams();
  const token = params.get("token");
  const t = useTranslations("auth.verify");
  const started = useRef(false);

  const verify = useMutation({
    mutationFn: (value: string) => authApi.verifyEmail(value),
  });

  useEffect(() => {
    if (!token || started.current) return;
    started.current = true;
    verify.mutate(token);
  }, [token, verify]);

  const succeeded = verify.isSuccess;
  const failed = verify.isError;

  return (
    <AuthCard
      title={t("title")}
      description={succeeded || failed ? undefined : t("description")}
      footer={
        <Button asChild variant="outline" className="w-full">
          <Link to="/login">{t("back")}</Link>
        </Button>
      }
    >
      <div className="flex flex-col gap-4">
        {failed ? (
          <FormAlert message={t("error")} />
        ) : succeeded ? (
          <FormAlert message={t("success")} variant="default" />
        ) : (
          <FormAlert message={t("description")} variant="default" />
        )}
      </div>
    </AuthCard>
  );
}
