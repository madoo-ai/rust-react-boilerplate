import { Outlet, useLocation } from "react-router-dom";

import { AppHeader } from "@/components/app-shell/app-header";
import { AppSidebar } from "@/components/app-shell/app-sidebar";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { useSession } from "@/lib/auth";
import type { SessionUser } from "@/types";

export function AppLayout() {
  const session = useSession();
  const location = useLocation();

  if (!session) {
    return null;
  }

  const user: SessionUser = session.user;

  return (
    <SidebarProvider>
      <AppSidebar user={user} />
      <SidebarInset>
        <AppHeader />
        <div className="flex flex-1 flex-col gap-6 app-canvas p-4 md:p-6 lg:p-8">
          <div key={location.pathname} className="motion-safe:animate-page-in">
            <Outlet />
          </div>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}
