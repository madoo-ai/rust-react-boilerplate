import { useQuery } from "@tanstack/react-query";
import {
  CheckCircleIcon as CheckCircle2,
  ChevronRightIcon,
  ClockIcon as Clock3,
  Key01Icon as KeyRound,
  ShieldTickIcon as ShieldCheck,
  Users01Icon as Users,
} from "@x1nx3r/altics-react-icons";
import { Link } from "react-router-dom";

import { PageHeader } from "@/components/common/page-header";
import { StatCard } from "@/components/common/stat-card";
import { DashboardCharts } from "@/components/dashboard/dashboard-charts";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { api } from "@/lib/api";
import { useSession } from "@/lib/auth";
import {
  chartDateLabel,
  formatDate,
  formatNumber,
  initials,
} from "@/lib/format";
import { useLocale, useTranslations } from "@/lib/i18n";
import type { DashboardData, UserStatus } from "@/types";

function statusVariant(status: UserStatus) {
  if (status === "active") return "default" as const;
  if (status === "suspended") return "destructive" as const;
  return "secondary" as const;
}

export function DashboardPage() {
  const session = useSession();
  const locale = useLocale();
  const t = useTranslations("dashboard");
  const tCommon = useTranslations("common");
  const tUserColumns = useTranslations("users.columns");

  const canViewAdmin =
    session?.user.permissions.includes("view_admin") ?? false;

  const dashboard = useQuery({
    queryKey: ["dashboard"],
    queryFn: () => api.get<DashboardData>("/dashboard"),
    enabled: canViewAdmin,
  });

  if (!session) return null;

  if (!canViewAdmin) {
    return (
      <div className="flex flex-col gap-6">
        <PageHeader
          title={t("title")}
          description={t("description")}
          actions={
            <Button asChild variant="outline">
              <Link to="/settings/profile">{tCommon("profile")}</Link>
            </Button>
          }
        />
        <Card className="relative gap-0 overflow-hidden">
          <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_top_left,color-mix(in_oklab,var(--primary)_14%,transparent),transparent_62%)]" />
          <CardContent className="relative flex flex-col gap-6 p-6 sm:flex-row sm:items-center">
            <Avatar className="size-16 rounded-2xl ring-1 ring-border/60">
              <AvatarFallback className="rounded-2xl bg-primary/10 text-xl text-primary">
                {initials(session.user.name)}
              </AvatarFallback>
            </Avatar>
            <div className="flex min-w-0 flex-1 flex-col gap-1">
              <p className="text-xs text-muted-foreground">
                {tCommon("profile")}
              </p>
              <p className="font-display text-3xl leading-tight tracking-tight">
                {session.user.name}
              </p>
              <p className="truncate text-sm text-muted-foreground">
                {session.user.email}
              </p>
            </div>
            <div className="flex flex-wrap gap-2">
              <Badge variant="outline">
                {session.user.role?.label ?? tCommon("unknown")}
              </Badge>
              <Badge variant={statusVariant(session.user.status)}>
                {tCommon(session.user.status)}
              </Badge>
            </div>
          </CardContent>
          <div className="relative grid gap-4 border-t border-border/60 p-6 sm:grid-cols-3">
            <div className="flex flex-col gap-1 text-sm">
              <span className="text-muted-foreground">
                {tCommon("jobTitle")}
              </span>
              <span>{session.user.jobTitle ?? "—"}</span>
            </div>
            <div className="flex flex-col gap-1 text-sm">
              <span className="text-muted-foreground">
                {tCommon("department")}
              </span>
              <span>{session.user.department ?? "—"}</span>
            </div>
            <div className="flex flex-col gap-1 text-sm">
              <span className="text-muted-foreground">{tCommon("phone")}</span>
              <span>{session.user.phoneNumber ?? "—"}</span>
            </div>
          </div>
        </Card>
        <div className="grid gap-4 sm:grid-cols-3">
          <StatCard
            label={tCommon("role")}
            value={session.user.role?.label ?? tCommon("unknown")}
            icon={ShieldCheck}
          />
          <StatCard
            label={tCommon("status")}
            value={tCommon(session.user.status)}
            icon={CheckCircle2}
          />
          <StatCard
            label={tCommon("security")}
            value={
              session.user.twoFactorEnabled
                ? tCommon("enabled")
                : tCommon("disabled")
            }
            icon={KeyRound}
          />
        </div>
        <div className="flex flex-wrap gap-2">
          <Button asChild variant="outline">
            <Link to="/settings/profile">{tCommon("profile")}</Link>
          </Button>
          <Button asChild variant="outline">
            <Link to="/settings/password">{tCommon("password")}</Link>
          </Button>
          <Button asChild variant="outline">
            <Link to="/settings/security">{tCommon("security")}</Link>
          </Button>
        </div>
      </div>
    );
  }

  const data = dashboard.data;

  if (!data) {
    return (
      <div className="flex flex-col gap-6">
        <PageHeader title={t("title")} description={t("description")} />
        <Card>
          <CardContent className="py-10 text-sm text-muted-foreground">
            {tCommon("loading")}
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-6">
      <PageHeader
        title={t("title")}
        description={t("description")}
        actions={
          <Button asChild>
            <Link to="/admin/users">{t("viewAll")}</Link>
          </Button>
        }
      />
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <StatCard
          label={t("stats.totalUsers")}
          value={formatNumber(data.totalUsers, locale)}
          icon={Users}
        />
        <StatCard
          label={t("stats.activeUsers")}
          value={formatNumber(data.activeUsers, locale)}
          icon={CheckCircle2}
        />
        <StatCard
          label={t("stats.pendingUsers")}
          value={formatNumber(data.pendingUsers, locale)}
          icon={Clock3}
        />
        <StatCard
          label={t("stats.roles")}
          value={formatNumber(data.rolesCount, locale)}
          icon={ShieldCheck}
        />
      </div>
      <DashboardCharts
        roleData={data.roleData}
        signupData={data.signupData.map((point) => ({
          date: chartDateLabel(point.date, locale),
          users: point.users,
        }))}
      />
      <Card>
        <CardHeader>
          <CardTitle>{t("recentUsers")}</CardTitle>
          <CardAction>
            <Button asChild variant="ghost" size="sm">
              <Link to="/admin/users">
                {t("viewAll")}
                <ChevronRightIcon data-icon="inline-end" />
              </Link>
            </Button>
          </CardAction>
        </CardHeader>
        <CardContent>
          {data.recentUsers.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              {t("recentUsersEmpty")}
            </p>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{tCommon("name")}</TableHead>
                  <TableHead>{tCommon("role")}</TableHead>
                  <TableHead>{tCommon("status")}</TableHead>
                  <TableHead className="text-right">
                    {tUserColumns("createdAt")}
                  </TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {data.recentUsers.map((user) => (
                  <TableRow key={user.id}>
                    <TableCell>
                      <div className="flex items-center gap-3">
                        <Avatar className="size-8">
                          <AvatarFallback className="bg-primary/10 text-xs text-primary">
                            {initials(user.name)}
                          </AvatarFallback>
                        </Avatar>
                        <div className="flex flex-col">
                          <span className="font-medium">{user.name}</span>
                          <span className="text-xs text-muted-foreground">
                            {user.email}
                          </span>
                        </div>
                      </div>
                    </TableCell>
                    <TableCell>
                      <Badge variant="outline">
                        {user.role?.label ?? tCommon("unknown")}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <Badge variant={statusVariant(user.status)}>
                        {tCommon(user.status)}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-right text-muted-foreground">
                      {formatDate(user.createdAt, locale)}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
