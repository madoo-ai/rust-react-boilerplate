import { Outlet } from "react-router-dom";

import { MarketingFooter } from "@/components/marketing/marketing-footer";
import { MarketingHeader } from "@/components/marketing/marketing-header";
import { useSession } from "@/lib/auth";

export function MarketingLayout() {
  const session = useSession();

  return (
    <div className="flex min-h-svh flex-col bg-background">
      <MarketingHeader isAuthenticated={!!session} />
      <main className="flex-1">
        <Outlet />
      </main>
      <MarketingFooter />
    </div>
  );
}
