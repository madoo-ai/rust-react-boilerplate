import { existsSync } from "node:fs";
import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const [displayName, slug] = process.argv.slice(2);

if (!displayName || !slug) {
  console.error('Usage: node scripts/rename.mjs "Client App" client-app');
  process.exit(1);
}

if (!/^[a-z0-9][a-z0-9-_]*$/.test(slug)) {
  console.error(
    "The slug must start with a lowercase letter or number and contain only lowercase letters, numbers, dashes or underscores.",
  );
  process.exit(1);
}

const root = resolve(import.meta.dirname, "..");
const oldPackage = "rust-react-boilerplate-api";
const newPackage = `${slug}-api`;

async function updateFile(path, transform) {
  const absolute = resolve(root, path);
  if (!existsSync(absolute)) return;

  const content = await readFile(absolute, "utf8");
  const next = transform(content);

  if (next !== content) {
    await writeFile(absolute, next, "utf8");
    console.log(`updated ${path}`);
  }
}

function replacePackageName(content) {
  return content.replaceAll(oldPackage, newPackage);
}


await updateFile("web/package.json", (content) => {
  const json = JSON.parse(content);
  json.name = `${slug}-web`;
  return `${JSON.stringify(json, null, 2)}\n`;
});

await updateFile("web/package-lock.json", (content) => {
  const json = JSON.parse(content);
  json.name = `${slug}-web`;
  if (json.packages?.[""]) json.packages[""].name = `${slug}-web`;
  return `${JSON.stringify(json, null, 2)}\n`;
});

await updateFile("web/src/config/app.ts", (content) =>
  content.replace(/"Rust React Boilerplate"/, JSON.stringify(displayName)),
);

for (const locale of ["en", "id"]) {
  await updateFile(`web/src/messages/${locale}.json`, (content) =>
    content.replace(/"appName": ".*"/, `"appName": ${JSON.stringify(displayName)}`),
  );
}

await updateFile(".env", (content) =>
  content.replace(/^APP_NAME=.*$/m, `APP_NAME=${JSON.stringify(displayName)}`),
);

await updateFile(".env.example", (content) =>
  content.replace(/^APP_NAME=.*$/m, `APP_NAME=${JSON.stringify(displayName)}`),
);

await updateFile("api/Cargo.toml", replacePackageName);
await updateFile("Cargo.lock", replacePackageName);
await updateFile("Makefile", replacePackageName);
await updateFile("Dockerfile", (content) =>
  replacePackageName(content).replace(
    /ARG VITE_APP_NAME=".*"/,
    `ARG VITE_APP_NAME=${JSON.stringify(displayName)}`,
  ),
);
await updateFile("scripts/e2e-server.sh", replacePackageName);
await updateFile("README.md", replacePackageName);
await updateFile("AGENTS.md", replacePackageName);
await updateFile(".github/workflows/ci.yml", replacePackageName);

await updateFile("README.md", (content) =>
  content.replace(/^# .+$/m, `# ${displayName}`),
);

await updateFile("AGENTS.md", (content) =>
  content.replace(/^# .+$/m, `# ${displayName} — agent guide`),
);

await updateFile("compose.yaml", (content) =>
  content
    .replace(/^name: .+$/m, `name: ${slug}`)
    .replace(/\$\{APP_NAME:-[^}]*\}/g, `\${APP_NAME:-${displayName}}`),
);

await updateFile("web/index.html", (content) =>
  content.replace(/<title>.*<\/title>/, `<title>${displayName}</title>`),
);

console.log(`Renamed to "${displayName}" (${slug}).`);
