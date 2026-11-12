import { useQuery } from "@tanstack/react-query";

import { PageHeader } from "@/components/common/page-header";
import { UserForm } from "@/components/users/user-form";
import { Card, CardContent } from "@/components/ui/card";
import { api } from "@/lib/api";
import { useTranslations } from "@/lib/i18n";
import type { UserMeta } from "@/types";

export function CreateUserPage() {
  const t = useTranslations("users");
  const tCommon = useTranslations("common");

  const meta = useQuery({
    queryKey: ["user-meta"],
    queryFn: () => api.get<UserMeta>("/users/meta"),
  });

  return (
    <div className="flex flex-col gap-6">
      <PageHeader title={t("create")} description={t("description")} />
      {meta.data ? (
        <UserForm mode="create" roles={meta.data.roles} />
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
