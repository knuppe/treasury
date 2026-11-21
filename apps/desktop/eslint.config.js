// @ts-check
import js from '@eslint/js';
import i18next from 'eslint-plugin-i18next';
import reactHooks from 'eslint-plugin-react-hooks';
import { defineConfig, globalIgnores } from 'eslint/config';
import tseslint from 'typescript-eslint';

/**
 * Pastas já passadas para os catálogos de idioma (docs/i18n-design.md): nelas, texto fixo em
 * JSX é erro. Cada fase acrescenta as pastas que migrou; as outras ainda têm texto em pt-BR.
 */
const I18N_DONE = [
  'src/i18n/**',
  'src/app/**',
  'src/features/agent/**',
  'src/features/companion/**',
  'src/charts/**',
  'src/commands/**',
  'src/content/**',
  'src/design/**',
  'src/vault/**',
  'src/features/accounts/**',
  'src/features/cards/**',
  'src/features/budget/**',
  'src/features/country/**',
  'src/features/dashboard/**',
  'src/features/demo/**',
  'src/features/fx/**',
  'src/features/health/**',
  'src/features/import/**',
  'src/features/income/**',
  'src/features/longevity/**',
  'src/features/mail/**',
  'src/features/portfolio/*.{ts,tsx}',
  'src/features/portfolio/charts/**',
  'src/features/portfolio/components/**',
  'src/features/planning/**',
  'src/features/profile/**',
  'src/features/reports/controls.tsx',
  'src/features/reports/explain/**',
  'src/features/rules/**',
  'src/features/settings/**',
  'src/features/support/**',
  'src/features/terms/**',
  'src/features/transactions/**',
  'src/screens/Agente.tsx',
  'src/screens/Ajustes.tsx',
  'src/screens/Carteira.tsx',
  'src/screens/Contas.tsx',
  'src/screens/Familia.tsx',
  'src/screens/Lancamentos.tsx',
  'src/screens/Orcamento.tsx',
  'src/screens/Painel.tsx',
  'src/screens/NotFound.tsx',
  // Fase 3 e 5 (Dívidas, Metas, Projetos, Bens, Ativos, Relatórios, Impostos, Desenvolvedor).
  'src/charts/InflationChart.tsx',
  'src/charts/inflationOption.ts',
  'src/features/assets/**',
  'src/features/debts/**',
  'src/features/devfeedback/**',
  'src/features/goals/**',
  'src/features/indices/**',
  'src/features/market/**',
  'src/features/physical/**',
  'src/features/portfolio/import/**',
  'src/features/projects/**',
  'src/features/receipts/**',
  'src/features/reports/**',
  'src/features/research/**',
  'src/features/score/**',
  'src/features/tax/**',
  'src/features/taxburden/**',
  'src/screens/Ativos.tsx',
  'src/screens/Desenvolvedor.tsx',
  'src/screens/Dividas.tsx',
  'src/screens/Impostos.tsx',
  'src/screens/Relatorios.tsx',
  'src/screens/dev/**',
  // Telas e pastas que vieram depois das fases (já pelo catálogo; o lint impede texto fixo novo).
  'src/features/food/**',
  'src/lib/**',
  'src/state/**',
  'src/screens/Ativo.tsx',
  'src/screens/Consumo.tsx',
  'src/screens/Mercado.tsx',
];

export default defineConfig([
  globalIgnores(['dist', 'coverage', 'src-tauri', 'src/api/generated', 'eslint.config.js']),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      js.configs.recommended,
      tseslint.configs.recommendedTypeChecked,
      reactHooks.configs.flat.recommended,
    ],
    languageOptions: {
      ecmaVersion: 2023,
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      '@typescript-eslint/consistent-type-imports': ['error', { fixStyle: 'inline-type-imports' }],
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_', caughtErrors: 'none' },
      ],
      '@typescript-eslint/no-floating-promises': 'error',
      '@typescript-eslint/no-misused-promises': [
        'error',
        { checksVoidReturn: { attributes: false } },
      ],
      eqeqeq: ['error', 'smart'],
      'no-console': ['warn', { allow: ['warn', 'error'] }],
    },
  },
  {
    files: I18N_DONE,
    ignores: ['src/**/*.test.{ts,tsx}'],
    plugins: { i18next },
    rules: { 'i18next/no-literal-string': 'error' },
  },
  {
    files: ['src/**/*.test.{ts,tsx}', 'src/test/**'],
    rules: {
      '@typescript-eslint/no-non-null-assertion': 'off',
      '@typescript-eslint/unbound-method': 'off',
    },
  },
]);
