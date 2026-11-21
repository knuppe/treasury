/**
 * Linha da vida de um membro (docs/life-expectancy.md, seção 6.1): as fases da vida em
 * segmentos com as cores do tema, preenchida até a idade de hoje e, depois dela, em tom suave;
 * o final esfumado entre a mediana, o "1 em 4" e o "1 em 10" (nunca uma parede); os marcos
 * financeiros como pinos com dica; o texto no tom do perfil e a fonte sempre visível. Crianças
 * mostram fases e marcos, e o final só com o ajuste. A barra tem uma alternativa em texto com
 * os mesmos números, e enche uma vez ao abrir (parada com "reduzir movimento").
 */
import './lifebar.css';

import { useState, type CSSProperties, type ReactNode } from 'react';

import type { LifeMilestone } from '@/api/generated/LifeMilestone';
import type { LongevitySource } from '@/api/generated/LongevitySource';
import type { MemberOutlook } from '@/api/generated/MemberOutlook';
import type { ProfileTone } from '@/api/generated/ProfileTone';
import type { StageInfo } from '@/api/generated/StageInfo';
import { sequentialRamp } from '@/charts/palette';
import { Money, Tooltip } from '@/design/components';
import { cx } from '@/design/cx';
import { prefersReducedMotion, type ThemeId } from '@/design/themes';
import { t } from '@/i18n';
import { useUi } from '@/state/ui';

import {
  ageAxis,
  altText,
  axisEndAge,
  clampPct,
  clusterMilestones,
  DEFAULT_STAGES,
  listText,
  milestoneKindLabel,
  outlookSentence,
  passAge,
  position,
  segments,
  stageAt,
  stageLabel,
  type Axis,
  type MilestoneKind,
  type Place,
  type Segment,
} from './model';

/** Cores das fases: a rampa sequencial do tema, do passo mais fraco ao mais forte. */
export function phaseColors(theme: ThemeId, n: number): string[] {
  const ramp = sequentialRamp(theme).filter((c) => c !== 'transparent');
  const start = n <= ramp.length - 1 ? 1 : 0;
  return Array.from({ length: n }, (_, i) => ramp[Math.min(ramp.length - 1, start + i)]!);
}

const KIND_COLOR: Record<MilestoneKind, string> = {
  retirement: 'var(--accent)',
  inss: 'var(--accent)',
  plan_end: 'var(--accent)',
  goal: 'var(--positive)',
  debt_end: 'var(--warning)',
  life_event: 'var(--text-dim)',
};

/** Máscara do final esfumado: opaca até a mediana, some depois do "1 em 10". */
function endMask(o: MemberOutlook, x: (age: number) => number, showEnd: boolean): string {
  if (!showEnd) return 'linear-gradient(to right, #000 0%, #000 90%, transparent 100%)';
  const m = x(o.median_age);
  const p25 = x(o.p25_age);
  const p10 = x(o.p10_age);
  const end = p10 + Math.max(3, (p10 - m) * 0.5);
  return `linear-gradient(to right, #000 0%, #000 ${m}%, rgb(0 0 0 / 0.6) ${p25}%, rgb(0 0 0 / 0.25) ${p10}%, transparent ${end}%)`;
}

function SegmentFills({
  segs,
  colors,
  x,
  opacity,
}: {
  segs: readonly Segment[];
  colors: readonly string[];
  x: (age: number) => number;
  opacity: number;
}) {
  return (
    <>
      {segs.map((s) => {
        const left = clampPct(x(s.from));
        const right = clampPct(x(s.to));
        if (right <= left) return null;
        return (
          <span
            key={`${s.key}-${s.from}`}
            className="absolute inset-y-0"
            style={{
              left: `${left}%`,
              width: `${right - left}%`,
              background: colors[s.index],
              opacity,
            }}
          />
        );
      })}
    </>
  );
}

function MilestoneTip({ items }: { items: readonly LifeMilestone[] }) {
  return (
    <div className="flex min-w-44 flex-col gap-2">
      {items.map((m) => {
        const kind = milestoneKindLabel(m.kind);
        return (
          <div key={`${m.kind}-${m.label}-${m.age}`} className="flex flex-col gap-0.5">
            <span className="label-caps" style={{ color: KIND_COLOR[m.kind] }}>
              {kind}
            </span>
            {/* O nome do marco (meta, dívida, evento); sem repetir o tipo. */}
            {m.label !== kind ? <span className="font-medium text-fg">{m.label}</span> : null}
            <span className="text-dim">
              {t('longevidade-marco-quando', { idade: Math.round(m.age), ano: String(m.year) })}
            </span>
            {m.amount != null ? <Money cents={m.amount} className="text-[12px]" /> : null}
          </div>
        );
      })}
    </div>
  );
}

export interface LifeBarProps {
  outlook: MemberOutlook;
  /** Tom do perfil de quem vê: o texto é simples, equilibrado ou técnico. */
  tone: ProfileTone;
  /** País da família (o texto diz "pelas médias do Brasil"). */
  place: Place;
  stages?: readonly StageInfo[];
  /** Eixo comum (vista da família); padrão: idade de 0 até passar do "1 em 10". */
  axis?: Axis;
  /** Subdivisão da longevidade avançada (80+). */
  advanced?: boolean;
  /** Marcos no lugar dos que vieram do núcleo (o planejador mostra o plano em edição). */
  milestones?: readonly LifeMilestone[];
  /** Fonte das tábuas (o texto técnico cita o nome). */
  source?: LongevitySource | null;
  /** Mostra a linha da fonte embaixo (a família mostra uma vez só, no fim do quadro). */
  showSource?: boolean;
  showHeader?: boolean;
  showText?: boolean;
  /** Nomes das fases em cima da barra (vista por idade). */
  showPhaseNames?: boolean;
  /** Idades das fases, de hoje e do final embaixo da barra (vista por idade). */
  showTicks?: boolean;
  /** Linha extra ao lado do cabeçalho. */
  note?: ReactNode;
  className?: string;
}

export function LifeBar({
  outlook: o,
  tone,
  place,
  stages = DEFAULT_STAGES,
  axis,
  advanced = false,
  milestones,
  source,
  showSource = true,
  showHeader = true,
  showText = true,
  showPhaseNames = true,
  showTicks = true,
  note,
  className,
}: LifeBarProps) {
  const theme = useUi((s) => s.theme);
  // Enche uma vez ao montar; com "reduzir movimento", já aparece cheia.
  const [motion] = useState<'fill' | 'static'>(() => (prefersReducedMotion() ? 'static' : 'fill'));
  const ax = axis ?? ageAxis([o]);
  const x = (age: number) => position(ax, o, age);
  const segs = segments(stages, axisEndAge(ax, o), advanced);
  const colors = phaseColors(theme, segs.length);
  const today = clampPct(x(o.age));
  const showEnd = o.show_end;
  const marks = (milestones ?? o.milestones).filter((m) => {
    const p = x(m.age);
    return p >= 0 && p <= 100;
  });
  const pins = clusterMilestones(marks.map((m) => ({ m, x: x(m.age) })));
  const mask = endMask(o, x, showEnd);
  const maskStyle: CSSProperties = { maskImage: mask, WebkitMaskImage: mask };
  const stage = stageLabel(stageAt(stages, o.age, advanced));

  // Embaixo da barra: o começo de cada fase, hoje e o final (sem encavalar).
  const todayLabelAt = Math.min(Math.max(today, 4), 94);
  const endAt = showEnd ? Math.min(clampPct(x(o.median_age)), 84) : 101;
  const bounds = segs
    .map((s) => s.from)
    .filter((a) => {
      const p = x(a);
      return p >= 0 && p <= 100 && Math.abs(p - todayLabelAt) > 6 && p < endAt - 2;
    });

  return (
    <div
      role="group"
      aria-label={t('longevidade-titulo-de', { nome: o.name })}
      data-motion={motion}
      className={cx('life-bar flex min-w-0 flex-col gap-1', className)}
    >
      {showHeader ? (
        <div className="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-0.5">
          <span className="text-[13px] text-fg">
            {t('longevidade-cabecalho', { nome: o.name, idade: o.age, fase: stage })}
          </span>
          {note ? <span className="text-[12px] text-dim">{note}</span> : null}
        </div>
      ) : null}
      {showPhaseNames && ax.mode === 'age' ? (
        <div aria-hidden className="relative h-4 text-[10.5px] text-dim">
          {segs.map((s) => {
            const left = clampPct(x(s.from));
            const width = clampPct(x(s.to)) - left;
            return width >= 11 ? (
              <span
                key={`${s.key}-${s.from}`}
                className="absolute top-0 truncate px-0.5"
                style={{ left: `${left}%`, width: `${width}%` }}
              >
                {stageLabel(s.key)}
              </span>
            ) : null;
          })}
        </div>
      ) : null}
      <div className="relative h-4">
        {pins.map((group) => {
          const items = group.map((g) => g.m);
          const first = items[0]!;
          const label =
            items.length === 1
              ? t('longevidade-marco-rotulo', {
                  rotulo: first.label,
                  idade: Math.round(first.age),
                  ano: String(first.year),
                })
              : t('longevidade-marcos-rotulo', {
                  idade: Math.round(first.age),
                  rotulos: listText(items.map((m) => m.label)),
                });
          return (
            <Tooltip
              key={`${first.kind}-${first.age}`}
              side="top"
              content={<MilestoneTip items={items} />}
            >
              <button
                type="button"
                aria-label={label}
                className="absolute bottom-0 flex h-4 min-w-4 -translate-x-1/2 items-center justify-center gap-0.5 rounded-[3px] focus-visible:outline-2 focus-visible:outline-accent"
                style={{ left: `${clampPct(group[0]!.x)}%` }}
              >
                <span
                  aria-hidden
                  className="size-2 rotate-45 rounded-[1px]"
                  style={{ background: KIND_COLOR[first.kind] }}
                />
                {items.length > 1 ? (
                  <span aria-hidden className="num text-[9.5px] text-dim">
                    {String(items.length)}
                  </span>
                ) : null}
              </button>
            </Tooltip>
          );
        })}
      </div>
      <div className="relative h-3">
        <div
          role="img"
          aria-label={altText(o, marks, advanced, stages)}
          className="life-bar-track absolute inset-0 overflow-hidden rounded-full bg-surface-2"
          style={maskStyle}
        >
          {/* Depois de hoje, em tom suave (na linha do tempo, tudo é futuro: um pouco mais forte). */}
          <SegmentFills
            segs={segs}
            colors={colors}
            x={x}
            opacity={ax.mode === 'year' ? 0.6 : 0.35}
          />
          {today > 0 ? (
            <div
              className="life-bar-fill absolute inset-0"
              style={{ clipPath: `inset(0 ${100 - today}% 0 0)` }}
            >
              <SegmentFills segs={segs} colors={colors} x={x} opacity={1} />
            </div>
          ) : null}
          {showEnd
            ? [o.median_age, o.p25_age, o.p10_age].map((a, i) => (
                <span
                  key={i}
                  className="absolute inset-y-0 w-px bg-fg/35"
                  style={{ left: `${clampPct(x(a))}%` }}
                />
              ))
            : null}
          <span className="life-bar-sweep" />
        </div>
        {today > 0 && today < 100 ? (
          <span
            aria-hidden
            className="absolute -top-1 -bottom-1 w-0.5 rounded-full bg-fg"
            style={{ left: `calc(${today}% - 1px)` }}
          />
        ) : null}
      </div>
      {showTicks && ax.mode === 'age' ? (
        <div aria-hidden className="num relative h-4 text-[10.5px] text-dim">
          {bounds.map((a) => (
            <span
              key={a}
              className={cx('absolute top-0', a > 0 && '-translate-x-1/2')}
              style={{ left: `${clampPct(x(a))}%` }}
            >
              {String(a)}
            </span>
          ))}
          {today > 0 ? (
            <span
              className="absolute top-0 -translate-x-1/2 whitespace-nowrap text-fg"
              style={{ left: `${todayLabelAt}%` }}
            >
              {t('longevidade-hoje', { idade: o.age })}
            </span>
          ) : null}
          {showEnd ? (
            <span
              className="absolute top-0 whitespace-nowrap"
              style={{ left: `${endAt}%` }}
              title={t('longevidade-final-dica', {
                mediana: passAge(o.median_age),
                p25: passAge(o.p25_age),
                p10: passAge(o.p10_age),
              })}
            >
              {t('longevidade-final-numeros', {
                mediana: passAge(o.median_age),
                p25: passAge(o.p25_age),
                p10: passAge(o.p10_age),
              })}
            </span>
          ) : null}
        </div>
      ) : null}
      {showText ? (
        <p className="text-[12.5px] leading-snug text-dim">
          {outlookSentence(o, tone, place, source?.name ?? '', advanced, stages)}
        </p>
      ) : null}
      {source && showSource ? <LifeSourceLine source={source} /> : null}
    </div>
  );
}

/** A fonte, sempre visível em uma linha (com o aviso dos dados sintéticos de desenvolvimento). */
export function LifeSourceLine({ source }: { source: LongevitySource }) {
  return (
    <p className="text-[11.5px] text-dim">
      {t('longevidade-fonte', { fonte: source.name })}
      {source.synthetic ? (
        <span className="ml-1 text-warning">{t('longevidade-fonte-sintetica')}</span>
      ) : null}
    </p>
  );
}

/** "Como calculamos": os limites da seção 4.5 e a atribuição das tábuas. */
export function LifeMethod({ source }: { source: LongevitySource }) {
  return (
    <details className="text-[12px] text-dim">
      <summary className="cursor-pointer text-fg">{t('longevidade-como')}</summary>
      <ul className="mt-1.5 flex list-disc flex-col gap-1 pl-4">
        <li>{t('longevidade-como-geracao')}</li>
        <li>{t('longevidade-como-idade')}</li>
        <li>{t('longevidade-como-limites')}</li>
        <li>{t('longevidade-como-futuro')}</li>
      </ul>
      <p className="mt-1.5 text-[11.5px]">{source.attribution}</p>
    </details>
  );
}
