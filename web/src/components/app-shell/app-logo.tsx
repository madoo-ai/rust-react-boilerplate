import { Link } from "react-router-dom";
import { cn } from "@/lib/utils";

import { APP_NAME } from "@/config/app";

export function AppLogoMark({ className }: { className?: string }) {
  return (
    <span
      className={cn(
        "grid size-8 shrink-0 place-items-center rounded-lg bg-primary text-primary-foreground shadow-[inset_0_1px_0_rgba(255,255,255,0.25)] ring-1 ring-primary/30",
        className,
      )}
      aria-hidden
    >
      <svg
        viewBox="0 0 24 24"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        className="size-4"
      >
        <path
          d="M6 18V6h4.6a4 4 0 0 1 0 8H8v4H6Zm2-6h2.6a2 2 0 0 0 0-4H8v4Z"
          fill="currentColor"
        />
      </svg>
    </span>
  );
}

export function AppLogo({ className }: { className?: string }) {
  return (
    <Link
      to="/"
      className={cn("flex items-center gap-2.5", className)}
      aria-label={APP_NAME}
    >
      <AppLogoMark />
      <span className="truncate text-sm font-semibold tracking-tight">
        {APP_NAME}
      </span>
    </Link>
  );
}
