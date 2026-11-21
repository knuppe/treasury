/** Rótulos e tons da saúde financeira (a forma vem de `health.score`). */
import type { HealthBand } from '@/api/generated/HealthBand';
import type { HealthModel } from '@/api/generated/HealthModel';
import type { HealthStatus } from '@/api/generated/HealthStatus';
import { t } from '@/i18n';
import { formatNumber } from '@/lib/format';

export const SOURCE_URL = 'https://meubolsoemdia.com.br/Materias/indice-saude-financeira-2024';

/** Nome do modelo no idioma do app (lido a cada acesso). */
export const MODEL_LABEL: Record<HealthModel, string> = {
  isf: 'I-SFB',
  get ratios() {
    return t('saude-modelo-classicos');
  },
  get off() {
    return t('saude-modelo-desativado');
  },
};

/** Estado: texto, sinal (nunca só a cor) e as classes de cor. */
export const STATUS: Record<
  HealthStatus,
  { readonly label: string; mark: string; text: string; chip: string; bar: string }
> = {
  good: {
    get label() {
      return t('saude-status-bom');
    },
    mark: '✓',
    text: 'text-positive',
    chip: 'border-positive/50 bg-positive-soft text-positive',
    bar: 'bg-positive',
  },
  ok: {
    get label() {
      return t('saude-status-atencao');
    },
    mark: '!',
    text: 'text-warning',
    chip: 'border-warning/50 bg-warning-soft text-warning',
    bar: 'bg-warning',
  },
  bad: {
    get label() {
      return t('saude-status-ruim');
    },
    mark: '▼',
    text: 'text-negative',
    chip: 'border-negative/50 bg-negative-soft text-negative',
    bar: 'bg-negative',
  },
  unknown: {
    get label() {
      return t('saude-status-sem-dados');
    },
    mark: '—',
    text: 'text-dim',
    chip: 'border-line-strong text-dim',
    bar: 'bg-line-strong',
  },
};

/** `62.4` → `62,4` (no idioma do app). */
export const formatScore = (s: number) => formatNumber(s, 1);

const GOOD_BANDS = new Set(['otima', 'muito_boa', 'saudavel']);
const OK_BANDS = new Set(['boa', 'ok', 'estavel']);

/** Tom da faixa: as de cima são boas, as do meio pedem atenção (como o núcleo). */
export function bandTone(band: HealthBand | null): HealthStatus {
  if (!band) return 'unknown';
  if (GOOD_BANDS.has(band.key)) return 'good';
  if (OK_BANDS.has(band.key)) return 'ok';
  return 'bad';
}

/** Cor de uma faixa na régua (da pior para a melhor). */
export function bandFill(band: HealthBand): string {
  const t = bandTone(band);
  return t === 'good' ? 'var(--positive)' : t === 'ok' ? 'var(--warning)' : 'var(--negative)';
}
