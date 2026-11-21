/**
 * Testes ponta a ponta: a interface (Vite) contra o núcleo Rust real (`treasury-dev serve`),
 * num cofre temporário. Rode com `npm run e2e` (compila o treasury-dev antes).
 *
 * O CORS do treasury-dev só libera http://localhost:1420, então o Vite usa essa porta.
 * Se `PW_CHROMIUM` estiver definido, usa esse executável (útil quando o Chromium do
 * Playwright não está baixado). `npm run e2e` roda no Chromium e `npm run e2e:webkit` no
 * WebKit, o motor da janela do app no macOS.
 */
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { defineConfig, devices } from '@playwright/test';

const API_PORT = 7799;
const TOKEN = 'e2e-token';
const HOME = join(tmpdir(), 'treasury-e2e');
const HERE = dirname(fileURLToPath(import.meta.url));
const DEV_BIN = join(HERE, '..', '..', 'target', 'debug', 'treasury-dev');

function chromiumPath(): string | undefined {
  return process.env.PW_CHROMIUM || undefined;
}

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  workers: 1,
  timeout: 60_000,
  expect: { timeout: 15_000 },
  reporter: [['list']],
  use: {
    baseURL: 'http://localhost:1420',
    locale: 'pt-BR',
    timezoneId: 'America/Sao_Paulo',
    trace: 'retain-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'], launchOptions: { executablePath: chromiumPath() } },
    },
    // O motor da janela do app no macOS (WKWebView). Roda em separado (`npm run e2e:webkit`),
    // porque os cenários compartilham um cofre só por execução.
    { name: 'webkit', use: { ...devices['Desktop Safari'] } },
  ],
  webServer: [
    {
      command: `rm -rf "${HOME}" && "${DEV_BIN}" serve --home "${HOME}" --port ${API_PORT} --token ${TOKEN} --no-mcp-socket`,
      url: `http://127.0.0.1:${API_PORT}/api/health`,
      reuseExistingServer: false,
      timeout: 60_000,
    },
    {
      command: 'npx vite --port 1420 --strictPort',
      url: 'http://localhost:1420',
      reuseExistingServer: false,
      timeout: 60_000,
      env: { VITE_TREASURY_API: `http://127.0.0.1:${API_PORT}`, VITE_TREASURY_TOKEN: TOKEN },
    },
  ],
});
