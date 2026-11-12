import { useQuery, useQueryClient } from "@tanstack/react-query";
import { createContext, useContext, type ReactNode } from "react";
import { Navigate, useLocation } from "react-router-dom";

import { api } from "@/lib/api";
import type { Permission, SessionData, SessionEnvelope } from "@/types";

type AuthContextValue = {
  session: SessionData | null;
  isLoading: boolean;
  refresh: () => Promise<unknown>;
  signOut: () => Promise<void>;
};

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ["session"],
    queryFn: () => api.get<SessionEnvelope>("/auth/session"),
    staleTime: 30_000,
    retry: false,
  });

  const session = query.data?.session ?? null;

  async function signOut() {
    try {
      await api.post("/auth/sign-out");
    } finally {
      queryClient.setQueryData<SessionEnvelope>(["session"], { session: null });
      queryClient.removeQueries({
        predicate: (item) => item.queryKey[0] !== "session",
      });
    }
  }

  return (
    <AuthContext
      value={{
        session,
        isLoading: query.isLoading,
        refresh: query.refetch,
        signOut,
      }}
    >
      {children}
    </AuthContext>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);

  if (!context) {
    throw new Error("useAuth must be used within AuthProvider");
  }

  return context;
}

export function useSession() {
  return useAuth().session;
}

export function RequireAuth({ children }: { children: ReactNode }) {
  const { session, isLoading } = useAuth();
  const location = useLocation();

  if (isLoading) {
    return <FullPageSpinner />;
  }

  if (!session) {
    const next = encodeURIComponent(`${location.pathname}${location.search}`);
    return <Navigate to={`/login?next=${next}`} replace />;
  }

  return children;
}

export function RequirePermission({
  permission,
  children,
}: {
  permission: Permission;
  children: ReactNode;
}) {
  const session = useSession();

  if (!session?.user.permissions.includes(permission)) {
    return <Navigate to="/403" replace />;
  }

  return children;
}

export function FullPageSpinner() {
  return (
    <div className="grid min-h-svh place-items-center bg-background">
      <div className="flex flex-col items-center gap-3 text-sm text-muted-foreground">
        <span className="size-6 animate-spin rounded-full border-2 border-primary/30 border-t-primary" />
        Loading...
      </div>
    </div>
  );
}
