import { Link } from "react-router-dom";

import { AuthCard } from "@/components/auth/auth-card";
import { TwoFactorForm } from "@/components/auth/two-factor-form";
import { useTranslations } from "@/lib/i18n";

export function TwoFactorPage() {
  const t = useTranslations("auth.twoFactor");

  return (
    <AuthCard
      title={t("title")}
      description={t("description")}
      footer={
        <Link
          to="/login"
          className="text-center font-medium text-primary underline-offset-4 hover:underline"
        >
          {t("backToLogin")}
        </Link>
      }
    >
      <TwoFactorForm />
    </AuthCard>
  );
}
