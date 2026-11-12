"use client";

import { Link, useLocation } from "react-router-dom";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useTranslations } from "@/lib/i18n";

const items = [
  { href: "/settings/profile", key: "profile" },
  { href: "/settings/password", key: "password" },
  { href: "/settings/appearance", key: "appearance" },
  { href: "/settings/security", key: "security" },
] as const;

export function SettingsNav() {
  const pathname = useLocation().pathname;
  const t = useTranslations("nav");

  return (
    <nav className="flex gap-1 overflow-x-auto lg:flex-col">
      {items.map((item) => {
        const active = pathname === item.href;

        return (
          <Button
            key={item.href}
            asChild
            variant="ghost"
            className={cn(
              "relative justify-start whitespace-nowrap",
              active &&
                "bg-primary/10 text-primary hover:bg-primary/15 hover:text-primary",
            )}
          >
            <Link to={item.href}>
              {active ? (
                <span className="absolute top-1/2 left-0 h-4 w-0.5 -translate-y-1/2 rounded-full bg-primary" />
              ) : null}
              {t(item.key)}
            </Link>
          </Button>
        );
      })}
    </nav>
  );
}
