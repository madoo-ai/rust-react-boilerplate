import { ApiError } from "@/lib/api";

type Translate = (key: string) => string;

export function errorMessage(error: unknown, t: Translate): string {
  if (error instanceof ApiError) {
    return translateApiError(error, t);
  }

  if (error instanceof Error && error.message) {
    return error.message;
  }

  return t("errors.unexpected");
}

export function authErrorMessage(error: unknown, t: Translate): string {
  if (error instanceof ApiError) {
    return translateApiError(error, t);
  }

  return errorMessage(error, t);
}

function translateApiError(error: ApiError, t: Translate): string {
  if (error.code === "ACCOUNT_PENDING") {
    return t("auth.messages.pendingApproval");
  }

  if (error.code === "ACCOUNT_SUSPENDED") {
    return t("auth.messages.suspended");
  }

  if (error.code === "INVALID_EMAIL_OR_PASSWORD") {
    return t("auth.messages.invalidCredentials");
  }

  if (error.code === "EMAIL_NOT_VERIFIED") {
    return t("auth.messages.emailNotVerified");
  }

  if (
    error.message.startsWith("errors.") ||
    error.message.startsWith("auth.") ||
    error.message.startsWith("validation.")
  ) {
    return t(error.message);
  }

  return error.message;
}
