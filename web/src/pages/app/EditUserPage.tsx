import { useQuery } from "@tanstack/react-query";
import { useParams } from "react-router-dom";

import { PageHeader } from "@/components/common/page-header";
import { UserForm } from "@/components/users/user-form";
import { Card, CardContent } from "@/components/ui/card";
import { FormAlert } from "@/components/auth/form-alert";
import { ApiError, api } from "@/lib/api";
import { errorMessage } from "@/lib/errors";
import { NotFoundPage } from "@/pages/NotFoundPage";
import { useTranslations } from "@/lib/i18n";
import type { EditableUser } from "@/types";

export function EditUserPage() {
  const { id } = useParams<{ id: string }>();
  const t = useTranslations("users");
  const tCommon = useTranslations("common");
  const tAll = useTranslations();

  const user = useQuery({
    queryKey: ["user", id],
    queryFn: () => api.get<EditableUser>(`/users/${id}`),
    enabled: !!id,
    retry: false,
  });

  if (user.error instanceof ApiError && user.error.status === 404) {
    return <NotFoundPage />;
  }

  return (
    <div className="flex flex-col gap-6">
      <PageHeader title={t("edit")} description={user.data?.user.email} />
      {user.data ? (
        <UserForm mode="edit" roles={user.data.roles} user={user.data.user} />
      ) : user.isError ? (
        <Card>
          <CardContent className="py-6">
            <FormAlert message={errorMessage(user.error, tAll)} />
          </CardContent>
        </Card>
      ) : (
        <Card>
          <CardContent className="py-10 text-sm text-muted-foreground">
            {tCommon("loading")}
          </CardContent>
        </Card>
      )}
    </div>
  );
}
