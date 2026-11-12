import { api } from "@/lib/api";
import type {
  SessionsResponse,
  SignInResponse,
  SignUpResponse,
  TotpSetup,
} from "@/types";

export const authApi = {
  signUp: (input: { name: string; email: string; password: string }) =>
    api.post<SignUpResponse>("/auth/sign-up/email", input),
  signIn: (input: { email: string; password: string; rememberMe: boolean }) =>
    api.post<SignInResponse>("/auth/sign-in/email", input),
  signOut: () => api.post("/auth/sign-out"),
  forgotPassword: (email: string) =>
    api.post("/auth/forgot-password", { email }),
  resetPassword: (token: string, newPassword: string) =>
    api.post("/auth/reset-password", { token, newPassword }),
  changePassword: (currentPassword: string, newPassword: string) =>
    api.post("/auth/change-password", { currentPassword, newPassword }),
  confirmPassword: (password: string) =>
    api.post("/auth/confirm-password", { password }),
  verifyEmail: (token: string) => api.post("/auth/verify-email", { token }),
  resendVerification: () => api.post("/auth/send-verification-email"),
  sessions: () => api.get<SessionsResponse>("/auth/sessions"),
  revokeSession: (id: string) => api.delete(`/auth/sessions/${id}`),
  revokeOtherSessions: () => api.post("/auth/sessions/revoke-others"),
  twoFactor: {
    enable: (password: string) =>
      api.post<TotpSetup>("/auth/two-factor/enable", { password }),
    verify: (code: string) => api.post("/auth/two-factor/verify", { code }),
    disable: (password: string) =>
      api.post("/auth/two-factor/disable", { password }),
    regenerateBackupCodes: (password: string) =>
      api.post<{ backupCodes: string[] }>("/auth/two-factor/backup-codes", {
        password,
      }),
    verifyTotp: (code: string, trustDevice: boolean) =>
      api.post<SignInResponse>("/auth/two-factor/verify-totp", {
        code,
        trustDevice,
      }),
    verifyBackupCode: (code: string, trustDevice: boolean) =>
      api.post<SignInResponse>("/auth/two-factor/verify-backup-code", {
        code,
        trustDevice,
      }),
  },
};
