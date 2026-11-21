/**
 * As linhas da vida da família (docs/life-expectancy.md, seção 6.2), em duas vistas:
 * "Por idade" (padrão: todas as barras no mesmo eixo de 0 a 100, para ver em que fase cada um
 * está) e "Linha do tempo" (o eixo vira o ano civil, de hoje em diante, com a aposentadoria do
 * titular marcada em todas as barras e os anos em que a chance de pelo menos um dos adultos
 * estar vivo cai a 25% e a 10%).
 */
import { useState } from 'react';

import type { FamilyOutlook } from '@/api/generated/FamilyOutlook';
import type { MemberOutlook } from '@/api/generated/MemberOutlook';
import type { ProfileTone } from '@/api/generated/ProfileTone';
import type { StageInfo } from '@/api/generated/StageInfo';
import { Segmented } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatPercent, todayIso } from '@/lib/format';
import { useUi } from '@/state/ui';

import { LifeBar, LifeSourceLine, phaseColors } from './LifeBar';
import {
  ageAxis,
  clampPct,
  DEFAULT_STAGES,
  segments,
  stageAt,
  stageLabel,
  yearAxis,
  type Axis,
  type Place,
} from './model';

type View = 'age' | 'year';

export interface FamilyLifeBarsProps {
  family: FamilyOutlook;
  /** Quem aparece (padrão: todos da família; "Ver como" mostra um só). */
  members?: readonly MemberOutlook[];
  tone: ProfileTone;
  place: Place;
  stages?: readonly StageInfo[];
  advanced?: boolean;
  /** A frase e as marcas do casal (pelo menos um dos dois vivo). */
  showCouple?: boolean;
}

function YearTicks({ axis }: { axis: Axis }) {
  const first = Math.ceil(axis.from / 10) * 10;
  const years: number[] = [];
  for (let y = first; y <= axis.to; y += 10) years.push(y);
  const at = (y: number) => ((y - axis.from) / (axis.to - axis.from)) * 100;
  return (
    <div
      aria-hidden
      title={t('longevidade-eixo-anos')}
      className="num relative h-4 text-[10.5px] text-dim"
    >
      <span className="absolute top-0 left-0">{String(axis.from)}</span>
      {years
        .filter((y) => at(y) > 8 && at(y) < 97)
        .map((y) => (
          <span key={y} className="absolute top-0 -translate-x-1/2" style={{ left: `${at(y)}%` }}>
            {String(y)}
          </span>
        ))}
    </div>
  );
}

/** Legenda das cores das fases (na linha do tempo, cada barra começa numa fase diferente). */
function PhaseLegend({ stages, advanced }: { stages: readonly StageInfo[]; advanced: boolean }) {
  const theme = useUi((s) => s.theme);
  const segs = segments(stages, 110, advanced);
  const colors = phaseColors(theme, segs.length);
  return (
    <ul className="flex flex-wrap gap-x-3 gap-y-1 text-[11.5px] text-dim">
      {segs.map((s) => (
        <li key={s.key} className="flex items-center gap-1.5">
          <span
            aria-hidden
            className="size-2.5 rounded-[3px]"
            style={{ background: colors[s.index] }}
          />
          {stageLabel(s.key)}
        </li>
      ))}
    </ul>
  );
}

export function FamilyLifeBars({
  family,
  members,
  tone,
  place,
  stages = DEFAULT_STAGES,
  advanced = false,
  showCouple = true,
}: FamilyLifeBarsProps) {
  const [view, setView] = useState<View>('age');
  const list = members ?? family.members;
  const thisYear = Number(todayIso().slice(0, 4));
  const axis = view === 'age' ? ageAxis(list) : yearAxis(list, thisYear);
  // A aposentadoria de quem tem o marco (o titular), marcada em todas as barras.
  const retiree = list.find((o) => o.milestones.some((m) => m.kind === 'retirement'));
  const retirement = retiree?.milestones.find((m) => m.kind === 'retirement');
  const couple =
    showCouple && family.any_alive_p25_year != null && family.any_alive_p10_year != null
      ? { p25: family.any_alive_p25_year, p10: family.any_alive_p10_year }
      : null;
  const at = (year: number) => clampPct(((year - axis.from) / (axis.to - axis.from)) * 100);
  const guides =
    view === 'year'
      ? [
          ...(retirement && retiree
            ? [{ year: retirement.year, label: `${retirement.label} · ${retirement.year}` }]
            : []),
          ...(couple
            ? [
                {
                  year: couple.p25,
                  label: t('longevidade-casal-marca', {
                    pct: formatPercent(0.25, { decimals: 0 }),
                    ano: String(couple.p25),
                  }),
                },
                {
                  year: couple.p10,
                  label: t('longevidade-casal-marca', {
                    pct: formatPercent(0.1, { decimals: 0 }),
                    ano: String(couple.p10),
                  }),
                },
              ]
            : []),
        ].filter((g) => g.year >= axis.from && g.year <= axis.to)
      : [];

  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <Segmented
          label={t('longevidade-vista')}
          size="sm"
          value={view}
          onChange={setView}
          options={[
            { value: 'age', label: t('longevidade-vista-idade') },
            { value: 'year', label: t('longevidade-vista-tempo') },
          ]}
        />
        {view === 'year' ? <PhaseLegend stages={stages} advanced={advanced} /> : null}
      </div>
      {view === 'year' ? <YearTicks axis={axis} /> : null}
      <div className="relative flex flex-col gap-4">
        {guides.map((g) => (
          <span
            key={`${g.year}-${g.label}`}
            aria-hidden
            title={g.label}
            className="pointer-events-none absolute inset-y-0 border-l border-dashed border-line-strong/70"
            style={{ left: `${at(g.year)}%` }}
          />
        ))}
        {list.map((o, i) => {
          // Na linha do tempo: a idade e a fase de cada um na aposentadoria do titular.
          const ageThen = retirement ? o.age + (retirement.year - thisYear) : null;
          const note =
            view === 'year' && retirement && retiree && ageThen != null && ageThen >= 0
              ? t('longevidade-tempo-na-aposentadoria', {
                  nome: retiree.name,
                  ano: String(retirement.year),
                  idade: ageThen,
                  fase: stageLabel(stageAt(stages, ageThen, advanced)),
                })
              : null;
          return (
            <LifeBar
              key={o.member_id}
              outlook={o}
              tone={tone}
              place={place}
              stages={stages}
              axis={axis}
              advanced={advanced}
              source={family.source}
              showSource={false}
              showText={view === 'age'}
              showPhaseNames={view === 'age' && i === 0}
              showTicks={view === 'age'}
              note={note}
            />
          );
        })}
      </div>
      {couple ? (
        <p className="text-[12.5px] text-dim">
          {t('longevidade-casal', { ano25: String(couple.p25), ano10: String(couple.p10) })}
        </p>
      ) : null}
      {list.some((o) => o.show_end) ? (
        <p className="text-[11.5px] text-dim">{t('longevidade-legenda-final')}</p>
      ) : null}
      <LifeSourceLine source={family.source} />
    </div>
  );
}
