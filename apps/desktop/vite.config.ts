/// <reference types="vitest/config" />
import { fileURLToPath, URL } from 'node:url';

import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { defineConfig, searchForWorkspaceRoot } from 'vite';

// Variáveis definidas pelo Tauri CLI durante `tauri dev` / `tauri build`.
const tauriDevHost = process.env.TAURI_DEV_HOST;
const tauriDebug = Boolean(process.env.TAURI_ENV_DEBUG);

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  // O Tauri mostra os próprios logs; não limpar a tela evita esconder erros do Rust.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: tauriDevHost ?? false,
    hmr: tauriDevHost ? { protocol: 'ws', host: tauriDevHost, port: 1421 } : undefined,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
    // O mock da estimativa de impostos lê os dados de referência do núcleo (`?raw`), e a
    // interface lê os catálogos de idioma da raiz do repositório.
    fs: {
      allow: [
        searchForWorkspaceRoot(process.cwd()),
        fileURLToPath(new URL('../../crates/treasury-core/src/tax_burden/data', import.meta.url)),
        // Catálogos de idioma, os mesmos do núcleo (docs/i18n-design.md).
        fileURLToPath(new URL('../../locales', import.meta.url)),
      ],
    },
  },
  envPrefix: ['VITE_', 'TAURI_ENV_'],
  build: {
    outDir: 'dist',
    // WKWebView (macOS) e WebKitGTK (Linux): Tailwind v4 já exige Safari 16.4+.
    target: ['es2022', 'safari16.4', 'chrome111'],
    minify: tauriDebug ? false : undefined,
    sourcemap: tauriDebug,
  },
  test: {
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
    // CSS não é processado nos testes, exceto tokens.css, motion.css, scanner.css, cinema.css,
    // activity.css, phosphor.css e lifebar.css (lidos como texto nos testes).
    css: {
      include: [
        /tokens\.css/,
        /motion\.css/,
        /scanner\.css/,
        /cinema\.css/,
        /activity\.css/,
        /phosphor\.css/,
        /lifebar\.css/,
      ],
    },
    restoreMocks: true,
    testTimeout: 20_000,
    unstubEnvs: true,
    unstubGlobals: true,
  },
});
