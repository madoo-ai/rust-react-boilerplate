import { Link } from "react-router-dom";

import { AuthCard } from "@/components/auth/auth-card";
import { ForgotPasswordForm } from "@/components/auth/forgot-password-form";
import { useTranslations } from "@/lib/i18n";

export function ForgotPasswordPage() {
  const t = useTranslations("auth.forgot");

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
      <ForgotPasswordForm />
    </AuthCard>
  );
}
