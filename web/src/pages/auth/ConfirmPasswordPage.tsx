import { useSearchParams } from "react-router-dom";

import { AuthCard } from "@/components/auth/auth-card";
import { ConfirmPasswordForm } from "@/components/auth/confirm-password-form";
import { useTranslations } from "@/lib/i18n";

export function ConfirmPasswordPage() {
  const [params] = useSearchParams();
  const t = useTranslations("auth.confirmPassword");

  return (
    <AuthCard title={t("title")} description={t("description")}>
      <ConfirmPasswordForm next={params.get("next") ?? undefined} />
    </AuthCard>
  );
}
