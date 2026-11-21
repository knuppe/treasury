/**
 * Peças comuns dos dados dos ativos com fonte: rótulos de campo, fonte e origem, o selo de
 * confiança e o selo da fonte (fonte · data · faixa).
 */
import type { FieldBand } from '@/api/generated/FieldBand';
import type { FieldCandidate } from '@/api/generated/FieldCandidate';
import type { FieldOrigin } from '@/api/generated/FieldOrigin';
import type { FieldState } from '@/api/generated/FieldState';
import type { FieldStatus } from '@/api/generated/FieldStatus';
import type { EnrichItem } from '@/api/generated/EnrichItem';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import type { MessageKey } from '@/i18n/keys';
import { formatDate } from '@/lib/format';

/** Chave dinâmica do catálogo; sem tradução, o texto de reserva. */
function dyn(key: string, fallback: string): string {
  const s = t(key as MessageKey & `dados-campo-sector`);
  return s === key ? fallback : s;
}

export const fieldLabel = (f: string) => dyn(`dados-campo-${f}`, f);
/** Valor de um campo para mostrar: alguns têm rótulo (o tipo do plano: `efpc` → "Fundo de
 * pensão"); os outros, como vieram. */
export const valueLabel = (f: string, v: string) => dyn(`dados-valor-${f}-${v}`, v);
export const sourceLabel = (s: string) => dyn(`dados-fonte-${s}`, t('dados-fonte-outra'));
export const originLabel = (o: FieldOrigin) => dyn(`dados-origem-${o}`, o);
export const stateLabel = (s: FieldState) => dyn(`dados-estado-${s}`, s);
export const bandLabel = (b: FieldBand) => dyn(`dados-faixa-${b}`, b);

function safeDate(iso: string | null): string | null {
  if (!iso) return null;
  try {
    return formatDate(iso);
  } catch {
    return iso;
  }
}

const BAND_TONE: Record<FieldBand, string> = {
  high: 'text-positive',
  medium: 'text-warning',
  low: 'text-negative',
};

const DOTS: Record<FieldBand, string> = { high: '●●●', medium: '●●○', low: '●○○' };

export function ConfidenceChip({ c }: { c: Pick<FieldCandidate, 'band' | 'confidence'> }) {
  return (
    <span className={cx('num inline-flex items-center gap-1 text-[12px]', BAND_TONE[c.band])}>
      <span aria-hidden>{DOTS[c.band]}</span>
      {t('dados-confianca', { faixa: bandLabel(c.band), valor: c.confidence })}
    </span>
  );
}

/** "CVM · FCA · 14/10/2026", com a ressalva e as fontes que concordam no título. */
export function SourceChip({ c }: { c: FieldCandidate }) {
  const date = safeDate(c.as_of);
  const extra = [
    c.note === 'matched_by_name' ? t('dados-achado-pelo-nome') : null,
    c.agreeing.length > 0
      ? t('dados-concordam', { fontes: c.agreeing.map(sourceLabel).join(', ') })
      : null,
  ].filter(Boolean);
  return (
    <span
      className="inline-flex flex-wrap items-baseline gap-x-1 text-[12px] text-dim"
      title={extra.join(' · ') || undefined}
    >
      <span className="text-fg">{sourceLabel(c.source)}</span>
      {date ? <span>· {date}</span> : null}
      {c.note === 'matched_by_name' ? <span className="text-warning">*</span> : null}
    </span>
  );
}

/** Selo da origem do valor atual ("você", "já preenchido", "CVM · FCA · alta"). */
export function OriginChip({ f }: { f: FieldStatus }) {
  if (!f.origin) return null;
  const base = 'rounded-sm border border-line px-1.5 py-px text-[11px] leading-tight';
  if (f.origin === 'enrichment' && f.current_source) {
    const date = safeDate(f.current_as_of);
    return (
      <span className={cx(base, 'text-dim')}>
        {[sourceLabel(f.current_source), date].filter(Boolean).join(' · ')}
        {f.current_confidence != null ? ` · ${f.current_confidence}` : ''}
      </span>
    );
  }
  return (
    <span
      className={cx(base, f.origin === 'user' ? 'text-accent' : 'text-dim')}
      title={f.origin === 'legacy' ? t('dados-origem-legacy-dica') : undefined}
    >
      {originLabel(f.origin)}
    </span>
  );
}

/** O item a gravar para uma sugestão (troca o valor do usuário só com `replace`). */
export function itemFor(assetId: string, f: FieldStatus, c: FieldCandidate): EnrichItem {
  const replace = f.current != null && (f.origin === 'user' || f.origin === 'legacy');
  return { asset_id: assetId, field: f.field, value: c.value, ...(replace ? { replace } : {}) };
}
