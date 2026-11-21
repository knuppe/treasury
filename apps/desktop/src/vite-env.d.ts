/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** URL do núcleo em modo HTTP (desenvolvimento fora do Tauri), ex.: `http://127.0.0.1:7420`. */
  readonly VITE_TREASURY_API?: string;
  /** Token de sessão enviado como `Authorization: Bearer …` no modo HTTP. */
  readonly VITE_TREASURY_TOKEN?: string;
  /** `1` força o núcleo falso em memória. */
  readonly VITE_TREASURY_MOCK?: string;
  /** Estado inicial do mock: `empty` (padrão), `locked` ou `unlocked`. */
  readonly VITE_TREASURY_MOCK_SEED?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
