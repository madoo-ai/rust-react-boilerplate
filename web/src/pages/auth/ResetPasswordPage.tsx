import { Link, useSearchParams } from "react-router-dom";

import { AuthCard } from "@/components/auth/auth-card";
import { ResetPasswordForm } from "@/components/auth/reset-password-form";
import { useTranslations } from "@/lib/i18n";

export function ResetPasswordPage() {
  const [params] = useSearchParams();
  const t = useTranslations("auth.reset");

  return (
    <AuthCard
      title={t("title")}
      description={t("description")}
      footer={
        <Link
          to="/login"
          className="text-center font-medium text-primary underline-offset-4 hover:underline"
        >
          {t("back")}
        </Link>
      }
    >
      <ResetPasswordForm token={params.get("token") ?? undefined} />
    </AuthCard>
  );
}
