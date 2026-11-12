"use client";

import { SearchLgIcon as Search, XIcon as X } from "@x1nx3r/altics-react-icons";
import { useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";

import { Button } from "@/components/ui/button";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
} from "@/components/ui/input-group";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useTranslations } from "@/lib/i18n";
import type { RoleOption } from "@/types";

type FilterValues = {
  search?: string;
  role?: string;
  status?: string;
};

export function UsersToolbar({
  roles,
  initialSearch,
  initialRole,
  initialStatus,
}: {
  roles: RoleOption[];
  initialSearch?: string;
  initialRole?: string;
  initialStatus?: string;
}) {
  const t = useTranslations("users");
  const tCommon = useTranslations("common");
  const navigate = useNavigate();
  const [params] = useSearchParams();
  const [search, setSearch] = useState(initialSearch ?? "");
  const [role, setRole] = useState(initialRole);
  const [status, setStatus] = useState(initialStatus);

  function applyFilters(overrides: FilterValues = {}) {
    const next: FilterValues = {
      search: search.trim() || undefined,
      role,
      status,
      ...overrides,
    };
    const nextParams = new URLSearchParams(params);

    for (const key of ["search", "role", "status", "page"]) {
      nextParams.delete(key);
    }

    if (next.search) nextParams.set("search", next.search);
    if (next.role) nextParams.set("role", next.role);
    if (next.status) nextParams.set("status", next.status);

    const query = nextParams.toString();
    navigate(query ? `/admin/users?${query}` : "/admin/users");
  }

  const hasFilters = !!search || !!role || !!status;

  return (
    <form
      className="flex flex-col gap-3 lg:flex-row lg:items-center"
      onSubmit={(event) => {
        event.preventDefault();
        applyFilters();
      }}
    >
      <InputGroup className="lg:max-w-sm">
        <InputGroupAddon>
          <Search />
        </InputGroupAddon>
        <InputGroupInput
          value={search}
          onChange={(event) => setSearch(event.target.value)}
          placeholder={t("searchPlaceholder")}
          aria-label={tCommon("search")}
        />
      </InputGroup>
      <div className="flex flex-col gap-2 sm:flex-row">
        <Select
          value={role ?? "all"}
          onValueChange={(value) => {
            const nextRole = value === "all" ? undefined : value;
            setRole(nextRole);
            applyFilters({ role: nextRole });
          }}
        >
          <SelectTrigger
            className="w-full sm:w-44"
            aria-label={t("filterRole")}
          >
            <SelectValue placeholder={t("filterRole")} />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              <SelectItem value="all">{tCommon("all")}</SelectItem>
              {roles.map((option) => (
                <SelectItem key={option.name} value={option.name}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectGroup>
          </SelectContent>
        </Select>
        <Select
          value={status ?? "all"}
          onValueChange={(value) => {
            const nextStatus = value === "all" ? undefined : value;
            setStatus(nextStatus);
            applyFilters({ status: nextStatus });
          }}
        >
          <SelectTrigger
            className="w-full sm:w-44"
            aria-label={t("filterStatus")}
          >
            <SelectValue placeholder={t("filterStatus")} />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              <SelectItem value="all">{tCommon("all")}</SelectItem>
              <SelectItem value="active">{tCommon("active")}</SelectItem>
              <SelectItem value="pending">{tCommon("pending")}</SelectItem>
              <SelectItem value="suspended">{tCommon("suspended")}</SelectItem>
            </SelectGroup>
          </SelectContent>
        </Select>
      </div>
      <div className="flex items-center gap-2">
        <Button type="submit" variant="secondary">
          <Search data-icon="inline-start" />
          {tCommon("search")}
        </Button>
        {hasFilters ? (
          <Button
            type="button"
            variant="ghost"
            onClick={() => {
              setSearch("");
              setRole(undefined);
              setStatus(undefined);
              navigate("/admin/users");
            }}
          >
            <X data-icon="inline-start" />
            {tCommon("remove")}
          </Button>
        ) : null}
      </div>
    </form>
  );
}
