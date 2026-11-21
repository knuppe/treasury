/** Peças comuns do card e da seção de saúde financeira: régua das faixas, selo e barras. */
import type { HealthBand } from '@/api/generated/HealthBand';
import type { HealthReference } from '@/api/generated/HealthReference';
import type { HealthStatus } from '@/api/generated/HealthStatus';
import { PRIVACY_MASK } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { usePrivacy } from '@/state/ui';

import { bandFill, formatScore, STATUS } from './model';

/** Selo de estado: sinal + texto (nunca só a cor). */
export function StatusChip({ status, className }: { status: HealthStatus; className?: string }) {
  const s = STATUS[status];
  return (
    <span
      className={cx(
        'label-caps inline-flex shrink-0 items-center gap-1 rounded-[3px] border px-1.5 py-px text-[10px]',
        s.chip,
        className,
      )}
    >
      <span aria-hidden>{s.mark}</span>
      {s.label}
    </span>
  );
}

/**
 * Régua de 0 a 100 com as faixas do modelo, o marcador da nota e a média de referência.
 * No modo privacidade a régua fica sem marcadores (a posição revelaria a nota).
 */
export function ScoreScale({
  bands,
  band,
  score,
  reference,
}: {
  bands: readonly HealthBand[];
  band: HealthBand | null;
  score: number | null;
  reference: HealthReference | null;
}) {
  const privacy = usePrivacy();
  if (bands.length === 0) return null;
  const pos = (v: number) => `${Math.min(100, Math.max(0, v))}%`;
  return (
    <div className="flex flex-col gap-1.5">
      <div className="relative pt-3">
        {!privacy && score != null ? (
          <span
            aria-hidden
            className="absolute top-0 -translate-x-1/2 text-[10px] leading-none text-fg"
            style={{ left: pos(score) }}
          >
            {'▼'}
          </span>
        ) : null}
        <div
          role="img"
          aria-label={t('saude-faixas-aria', {
            faixas: bands.map((b) => `${b.label} ${b.min}–${b.max}`).join(', '),
          })}
          className="flex h-2 w-full gap-px overflow-hidden rounded-[2px]"
        >
          {bands.map((b) => (
            <span
              key={b.key}
              title={`${b.label}: ${b.min}–${b.max}`}
              className="h-full"
              style={{
                width: `${((b.max - b.min + 1) / 101) * 100}%`,
                background: bandFill(b),
                opacity: band?.key === b.key ? 1 : 0.28,
              }}
            />
          ))}
        </div>
        {!privacy && reference ? (
          <span
            aria-hidden
            className="absolute top-2 bottom-[-3px] w-px border-l border-dashed border-fg/70"
            style={{ left: pos(reference.score) }}
          />
        ) : null}
      </div>
      <div className="num flex justify-between text-[10.5px] text-dim">
        <span>{'0'}</span>
        {reference ? (
          <span>
            {reference.short_label}: {formatScore(reference.score)}
          </span>
        ) : null}
        <span>{'100'}</span>
      </div>
    </div>
  );
}

/** Barra de 0 a 100 de uma dimensão ou de um indicador (vazia sem dados ou em privacidade). */
export function PointsBar({
  points,
  status,
  label,
}: {
  points: number | null;
  status: HealthStatus;
  label: string;
}) {
  const privacy = usePrivacy();
  return (
    <span
      role="img"
      aria-label={
        points == null
          ? t('saude-barra-sem-dados', { rotulo: label })
          : privacy
            ? t('saude-barra-oculto', { rotulo: label })
            : t('saude-barra-pontos', { rotulo: label, pontos: formatScore(points) })
      }
      className="block h-1.5 w-full overflow-hidden rounded-[2px] bg-line/50"
    >
      {points != null && !privacy ? (
        <span
          className={cx('block h-full rounded-r-[3px]', STATUS[status].bar)}
          style={{ width: `${Math.max(2, Math.min(100, points))}%` }}
        />
      ) : null}
    </span>
  );
}

/** Texto com a nota (o resumo): no modo privacidade, os números viram `•••••`. */
export function PrivateText({ text, className }: { text: string; className?: string }) {
  const privacy = usePrivacy();
  return (
    <p className={className}>{privacy ? text.replace(/\d+(?:[.,]\d+)?/g, PRIVACY_MASK) : text}</p>
  );
}
