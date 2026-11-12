const API_PREFIX = "/api";

type ErrorPayload = {
  error?: {
    code?: string;
    message?: string;
    fields?: Record<string, string>;
  };
};

export class ApiError extends Error {
  readonly status: number;
  readonly code: string;
  readonly fields?: Record<string, string>;

  constructor(
    status: number,
    code: string,
    message: string,
    fields?: Record<string, string>,
  ) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.code = code;
    this.fields = fields;
  }
}

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  const headers = new Headers(init.headers);
  headers.set("Accept", "application/json");

  if (init.body !== undefined && !headers.has("Content-Type")) {
    headers.set("Content-Type", "application/json");
  }

  const response = await fetch(`${API_PREFIX}${path}`, {
    ...init,
    headers,
    credentials: "same-origin",
  });

  if (!response.ok) {
    let payload: ErrorPayload | null = null;

    try {
      payload = (await response.json()) as ErrorPayload;
    } catch {
      // The response was not JSON; fall back to a generic error key.
    }

    throw new ApiError(
      response.status,
      payload?.error?.code ?? "request_failed",
      payload?.error?.message ?? "errors.unexpected",
      payload?.error?.fields,
    );
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return (await response.json()) as T;
}

function body(value: unknown): RequestInit {
  return { body: JSON.stringify(value) };
}

export const api = {
  get: <T>(path: string) => request<T>(path),
  post: <T>(path: string, value?: unknown) =>
    request<T>(path, {
      method: "POST",
      ...(value === undefined ? {} : body(value)),
    }),
  put: <T>(path: string, value?: unknown) =>
    request<T>(path, {
      method: "PUT",
      ...(value === undefined ? {} : body(value)),
    }),
  patch: <T>(path: string, value?: unknown) =>
    request<T>(path, {
      method: "PATCH",
      ...(value === undefined ? {} : body(value)),
    }),
  delete: <T>(path: string) => request<T>(path, { method: "DELETE" }),
};
