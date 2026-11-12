import { Link, Navigate } from "react-router-dom";

import { AuthCard } from "@/components/auth/auth-card";
import { RegisterForm } from "@/components/auth/register-form";
import { useSession } from "@/lib/auth";
import { useTranslations } from "@/lib/i18n";

export function RegisterPage() {
  const session = useSession();
  const t = useTranslations("auth.register");

  if (session) {
    return <Navigate to="/dashboard" replace />;
  }

  return (
    <AuthCard
      title={t("title")}
      description={t("description")}
      footer={
        <p className="text-center">
          {t("hasAccount")}{" "}
          <Link
            to="/login"
            className="font-medium text-primary underline-offset-4 hover:underline"
          >
            {t("login")}
          </Link>
        </p>
      }
    >
      <RegisterForm />
    </AuthCard>
  );
}
