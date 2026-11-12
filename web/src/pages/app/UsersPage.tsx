import { useQuery } from "@tanstack/react-query";
import { PlusIcon as Plus } from "@x1nx3r/altics-react-icons";
import { Link, useSearchParams } from "react-router-dom";

import { DataPagination } from "@/components/common/data-pagination";
import { PageHeader } from "@/components/common/page-header";
import { RegistrationApprovalCard } from "@/components/users/registration-approval-card";
import { UsersTable } from "@/components/users/users-table";
import { UsersToolbar } from "@/components/users/users-toolbar";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { api } from "@/lib/api";
import { useTranslations } from "@/lib/i18n";
import type { UserListResponse, UserMeta } from "@/types";

const PAGE_SIZE = 10;

export function UsersPage() {
  const [params] = useSearchParams();
  const t = useTranslations("users");
  const tCommon = useTranslations("common");

  const page = Number(params.get("page") ?? "1") || 1;
  const search = params.get("search") ?? undefined;
  const role = params.get("role") ?? undefined;
  const status = params.get("status") ?? undefined;

  const query = new URLSearchParams({ page: String(page) });
  if (search) query.set("search", search);
  if (role) query.set("role", role);
  if (status) query.set("status", status);

  const users = useQuery({
    queryKey: ["users", page, search, role, status],
    queryFn: () => api.get<UserListResponse>(`/users?${query.toString()}`),
  });

  const meta = useQuery({
    queryKey: ["user-meta"],
    queryFn: () => api.get<UserMeta>("/users/meta"),
  });

  return (
    <div className="flex flex-col gap-6">
      <PageHeader
        title={t("title")}
        description={t("description")}
        actions={
          <Button asChild>
            <Link to="/admin/users/create">
              <Plus data-icon="inline-start" />
              {t("create")}
            </Link>
          </Button>
        }
      />
      {meta.data ? (
        <RegistrationApprovalCard
          key={meta.data.registrationApproval ? "approval-on" : "approval-off"}
          enabled={meta.data.registrationApproval}
        />
      ) : null}
      <div className="flex flex-col gap-3">
        <UsersToolbar
          roles={meta.data?.roles ?? []}
          initialSearch={search}
          initialRole={role}
          initialStatus={status}
        />
        {users.data ? (
          <>
            <UsersTable users={users.data.users} />
            <DataPagination
              page={users.data.page}
              pageCount={users.data.pageCount}
              total={users.data.total}
              pageSize={PAGE_SIZE}
              basePath="/admin/users"
              query={{ search, role, status }}
            />
          </>
        ) : (
          <Card>
            <CardContent className="py-10 text-sm text-muted-foreground">
              {tCommon("loading")}
            </CardContent>
          </Card>
        )}
      </div>
    </div>
  );
}
