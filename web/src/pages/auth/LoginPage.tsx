import { Link, Navigate, useSearchParams } from "react-router-dom";

import { AuthCard } from "@/components/auth/auth-card";
import { LoginForm } from "@/components/auth/login-form";
import { useSession } from "@/lib/auth";
import { useTranslations } from "@/lib/i18n";

export function LoginPage() {
  const session = useSession();
  const [params] = useSearchParams();
  const t = useTranslations("auth.login");
  const tMessages = useTranslations("auth.messages");

  if (session) {
    return <Navigate to="/dashboard" replace />;
  }

  const error = params.get("error") ?? undefined;
  const notice = params.get("deleted")
    ? tMessages("accountDeleted")
    : params.get("reset")
      ? tMessages("passwordReset")
      : undefined;

  return (
    <AuthCard
      title={t("title")}
      description={t("description")}
      footer={
        <p className="text-center">
          {t("noAccount")}{" "}
          <Link
            to="/register"
            className="font-medium text-primary underline-offset-4 hover:underline"
          >
            {t("register")}
          </Link>
        </p>
      }
    >
      <LoginForm
        next={params.get("next") ?? undefined}
        initialError={error}
        notice={notice}
      />
    </AuthCard>
  );
}
