import { ErrorBoundary } from "@/components/common/error-boundary";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Toaster } from "@/components/ui/sonner";
import { ThemeProvider } from "@/components/providers/theme-provider";
import { AuthProvider, RequireAuth, RequirePermission } from "@/lib/auth";
import { I18nProvider } from "@/lib/i18n";
import { AppLayout } from "@/layouts/AppLayout";
import { AuthLayout } from "@/layouts/AuthLayout";
import { MarketingLayout } from "@/layouts/MarketingLayout";
import { LandingPage } from "@/pages/LandingPage";
import { ConfirmPasswordPage } from "@/pages/auth/ConfirmPasswordPage";
import { ForgotPasswordPage } from "@/pages/auth/ForgotPasswordPage";
import { LoginPage } from "@/pages/auth/LoginPage";
import { RegisterPage } from "@/pages/auth/RegisterPage";
import { ResetPasswordPage } from "@/pages/auth/ResetPasswordPage";
import { TwoFactorPage } from "@/pages/auth/TwoFactorPage";
import { VerifyEmailPage } from "@/pages/auth/VerifyEmailPage";
import { CreateUserPage } from "@/pages/app/CreateUserPage";
import { DashboardPage } from "@/pages/app/DashboardPage";
import { EditUserPage } from "@/pages/app/EditUserPage";
import { RolesPage } from "@/pages/app/RolesPage";
import { UsersPage } from "@/pages/app/UsersPage";
import { AppearanceSettingsPage } from "@/pages/settings/AppearanceSettingsPage";
import { PasswordSettingsPage } from "@/pages/settings/PasswordSettingsPage";
import { ProfileSettingsPage } from "@/pages/settings/ProfileSettingsPage";
import { SecuritySettingsPage } from "@/pages/settings/SecuritySettingsPage";
import { SettingsLayout } from "@/pages/settings/SettingsLayout";
import { ForbiddenPage } from "@/pages/ForbiddenPage";
import { NotFoundPage } from "@/pages/NotFoundPage";
import { Navigate, Route, Routes } from "react-router-dom";

export function App() {
  return (
    <I18nProvider>
      <ThemeProvider>
        <AuthProvider>
          <TooltipProvider>
            <ErrorBoundary>
              <Routes>
                <Route element={<MarketingLayout />}>
                  <Route path="/" element={<LandingPage />} />
                </Route>

                <Route element={<AuthLayout />}>
                  <Route path="/login" element={<LoginPage />} />
                  <Route path="/register" element={<RegisterPage />} />
                  <Route
                    path="/forgot-password"
                    element={<ForgotPasswordPage />}
                  />
                  <Route
                    path="/reset-password"
                    element={<ResetPasswordPage />}
                  />
                  <Route path="/verify-email" element={<VerifyEmailPage />} />
                  <Route path="/two-factor" element={<TwoFactorPage />} />
                  <Route
                    path="/confirm-password"
                    element={
                      <RequireAuth>
                        <ConfirmPasswordPage />
                      </RequireAuth>
                    }
                  />
                </Route>

                <Route
                  element={
                    <RequireAuth>
                      <AppLayout />
                    </RequireAuth>
                  }
                >
                  <Route path="/dashboard" element={<DashboardPage />} />
                  <Route
                    path="/admin/users"
                    element={
                      <RequirePermission permission="manage_users">
                        <UsersPage />
                      </RequirePermission>
                    }
                  />
                  <Route
                    path="/admin/users/create"
                    element={
                      <RequirePermission permission="manage_users">
                        <CreateUserPage />
                      </RequirePermission>
                    }
                  />
                  <Route
                    path="/admin/users/:id/edit"
                    element={
                      <RequirePermission permission="manage_users">
                        <EditUserPage />
                      </RequirePermission>
                    }
                  />
                  <Route
                    path="/admin/roles"
                    element={
                      <RequirePermission permission="manage_roles">
                        <RolesPage />
                      </RequirePermission>
                    }
                  />
                  <Route path="/settings" element={<SettingsLayout />}>
                    <Route
                      index
                      element={<Navigate to="/settings/profile" replace />}
                    />
                    <Route path="profile" element={<ProfileSettingsPage />} />
                    <Route path="password" element={<PasswordSettingsPage />} />
                    <Route
                      path="appearance"
                      element={<AppearanceSettingsPage />}
                    />
                    <Route path="security" element={<SecuritySettingsPage />} />
                  </Route>
                </Route>

                <Route path="/403" element={<ForbiddenPage />} />
                <Route path="*" element={<NotFoundPage />} />
              </Routes>
            </ErrorBoundary>
            <Toaster richColors position="top-right" closeButton />
          </TooltipProvider>
        </AuthProvider>
      </ThemeProvider>
    </I18nProvider>
  );
}
