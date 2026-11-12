"use client";

import {
  Area,
  AreaChart,
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  XAxis,
  YAxis,
} from "recharts";
import { useTranslations } from "@/lib/i18n";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from "@/components/ui/chart";

type RoleDatum = {
  name: string;
  label: string;
  users: number;
};

type SignupDatum = {
  date: string;
  users: number;
};

export function DashboardCharts({
  roleData,
  signupData,
}: {
  roleData: RoleDatum[];
  signupData: SignupDatum[];
}) {
  const t = useTranslations("dashboard.charts");
  const roleChartConfig = {
    users: {
      label: t("users"),
      color: "var(--chart-1)",
    },
  } satisfies ChartConfig;
  const signupChartConfig = {
    users: {
      label: t("users"),
      color: "var(--chart-1)",
    },
  } satisfies ChartConfig;

  return (
    <div className="grid gap-6 lg:grid-cols-2">
      <Card>
        <CardHeader>
          <CardTitle>{t("roleDistribution")}</CardTitle>
          <CardDescription>{t("roleDistributionHint")}</CardDescription>
        </CardHeader>
        <CardContent>
          <ChartContainer
            config={roleChartConfig}
            className="min-h-[220px] w-full"
          >
            <BarChart accessibilityLayer data={roleData}>
              <CartesianGrid vertical={false} />
              <XAxis
                dataKey="label"
                tickLine={false}
                tickMargin={10}
                axisLine={false}
              />
              <YAxis allowDecimals={false} tickLine={false} axisLine={false} />
              <ChartTooltip content={<ChartTooltipContent />} />
              <Bar dataKey="users" radius={6}>
                {roleData.map((role, index) => (
                  <Cell
                    key={role.name}
                    fill={`var(--chart-${(index % 5) + 1})`}
                  />
                ))}
              </Bar>
            </BarChart>
          </ChartContainer>
        </CardContent>
      </Card>
      <Card>
        <CardHeader>
          <CardTitle>{t("signups")}</CardTitle>
          <CardDescription>{t("signupsHint")}</CardDescription>
        </CardHeader>
        <CardContent>
          <ChartContainer
            config={signupChartConfig}
            className="min-h-[220px] w-full"
          >
            <AreaChart accessibilityLayer data={signupData}>
              <defs>
                <linearGradient id="fillUsers" x1="0" y1="0" x2="0" y2="1">
                  <stop
                    offset="5%"
                    stopColor="var(--color-users)"
                    stopOpacity={0.35}
                  />
                  <stop
                    offset="95%"
                    stopColor="var(--color-users)"
                    stopOpacity={0.02}
                  />
                </linearGradient>
              </defs>
              <CartesianGrid vertical={false} />
              <XAxis
                dataKey="date"
                tickLine={false}
                tickMargin={10}
                axisLine={false}
                minTickGap={24}
              />
              <YAxis allowDecimals={false} tickLine={false} axisLine={false} />
              <ChartTooltip content={<ChartTooltipContent />} />
              <Area
                dataKey="users"
                type="monotone"
                fill="url(#fillUsers)"
                stroke="var(--color-users)"
                strokeWidth={2}
              />
            </AreaChart>
          </ChartContainer>
        </CardContent>
      </Card>
    </div>
  );
}
