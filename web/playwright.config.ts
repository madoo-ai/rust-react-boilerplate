import { defineConfig } from "@playwright/test";

const testDatabaseUrl =
  process.env.TEST_DATABASE_URL ??
  "postgresql://postgres:postgres@127.0.0.1:5432/rust_boilerplate_test";

const apiEnv = {
  DATABASE_URL: testDatabaseUrl,
  APP_URL: "http://localhost:3100",
  APP_NAME: "Rust React Boilerplate",
  APP_LOCALE: "id",
  APP_SECRET: "e2e-secret-that-is-at-least-32-characters-long",
  APP_TIME_ZONE: "Asia/Jakarta",
  PORT: "3100",
  WEB_DIST: "web/dist",
  COOKIE_SECURE: "false",
  ALLOWED_ORIGINS: "http://localhost:5173,http://localhost:3100",
  RATE_LIMIT: "false",
  RUN_MIGRATIONS: "true",
  SMTP_HOST: "",
  MAIL_FROM: "hello@example.com",
  RUST_LOG: "warn",
};

export default defineConfig({
  testDir: "./tests/e2e",
  fullyParallel: false,
  workers: 1,
  retries: 0,
  timeout: 30_000,
  reporter: [["list"]],
  use: {
    baseURL: "http://localhost:5173",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
  },
  webServer: [
    {
      command: "bash scripts/e2e-server.sh",
      cwd: "..",
      url: "http://127.0.0.1:3100/api/health",
      reuseExistingServer: !process.env.CI,
      timeout: 180_000,
      env: apiEnv,
    },
    {
      command:
        "VITE_API_PROXY_TARGET=http://127.0.0.1:3100 npm run dev -- --host 127.0.0.1 --port 5173",
      url: "http://127.0.0.1:5173",
      reuseExistingServer: !process.env.CI,
      timeout: 120_000,
    },
  ],
});
