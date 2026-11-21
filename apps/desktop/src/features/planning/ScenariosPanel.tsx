/**
 * Cenários "e se" (docs/scenarios.md): eventos de vida (comprar imóvel, ter um filho, pausar
 * a carreira, mudar de renda, um gasto único ou um projeto) contra o cenário base, com o
 * patrimônio ano a ano, os próximos 12 meses da previsão de saldo e a aposentadoria.
 */
import { StudyNotice } from '@/features/terms/Disclaimer';
import { useCallback, useMemo, useState } from 'react';

import type { ScenarioEvent } from '@/api/generated/ScenarioEvent';
import type { ScenarioResult } from '@/api/generated/ScenarioResult';
import {
  useDeleteScenario,
  useSaveScenario,
  useSavedScenarios,
  useScenario,
} from '@/api/hooks/planning';
import { useProjects } from '@/api/hooks/projects';
import {
  ChartFrame,
  EChart,
  baseOption,
  categoryAxis,
  compactMoney,
  tooltipMoney,
  valueAxis,
  type ChartTheme,
} from '@/charts';
import {
  Button,
  DateInput,
  Field,
  Icon,
  IconButton,
  Input,
  InstrumentCard,
  Money,
  MoneyInput,
  Select,
  TechTable,
  toast,
} from '@/design/components';
import { tRich } from '@/features/settings/richText';
import { InlineError, Segmented } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { useSettleFlash } from '@/state/activity';
import { formatDate, formatLocale, formatMoney, formatMonthShort, todayIso } from '@/lib/format';
import { monthOf, shiftMonth } from '@/lib/months';

type Preset = 'imovel' | 'filho' | 'pausa' | 'renda' | 'unico' | 'projeto';

/** Eventos de vida do seletor, no idioma do app. */
function presets(): { value: Preset; label: string }[] {
  return [
    { value: 'imovel', label: t('planejamento-cenarios-imovel') },
    { value: 'filho', label: t('planejamento-cenarios-filho') },
    { value: 'pausa', label: t('planejamento-cenarios-pausa') },
    { value: 'renda', label: t('planejamento-cenarios-renda') },
    { value: 'unico', label: t('planejamento-cenarios-unico') },
    { value: 'projeto', label: t('planejamento-cenarios-projeto') },
  ];
}

/** Número digitado (`1.234,5`; em inglês, `1,234.5`). */
const num = (s: string) =>
  (formatLocale() === 'en'
    ? Number(s.replace(/,/g, ''))
    : Number(s.replace(/\./g, '').replace(',', '.'))) || 0;

function describe(e: ScenarioEvent): string {
  const m = (c: number) => formatMoney(c);
  switch (e.kind) {
    case 'one_off':
      return t('planejamento-cenarios-evento-unico', {
        valor: m(e.amount),
        data: formatDate(e.date),
      });
    case 'monthly':
      return t('planejamento-cenarios-evento-mensal', {
        valor: m(e.amount),
        inicio: formatDate(e.start),
        meses: e.months ?? 0,
      });
    case 'income_pct':
      return t('planejamento-cenarios-evento-renda', {
        pct: e.pct,
        inicio: formatDate(e.start),
        meses: e.months ?? 0,
      });
    case 'asset':
      return t('planejamento-cenarios-evento-bem', {
        valor: m(e.value),
        data: formatDate(e.date),
        taxa: e.real_growth_pct ?? 0,
      });
    case 'loan':
      return t('planejamento-cenarios-evento-financiamento', {
        valor: m(e.principal),
        meses: e.months,
        taxa: e.annual_rate_pct,
      });
  }
}

/** Formulário do evento escolhido; devolve os eventos ao adicionar. */
function PresetForm({ preset, onAdd }: { preset: Preset; onAdd: (e: ScenarioEvent[]) => void }) {
  const next = `${shiftMonth(monthOf(todayIso()), 1)}-01`;
  const [date, setDate] = useState<string | null>(next);
  const [value, setValue] = useState<number | null>(null);
  const [down, setDown] = useState('20');
  const [years, setYears] = useState('30');
  const [rate, setRate] = useState('5');
  const [months, setMonths] = useState('12');
  const [pct, setPct] = useState('0');
  const [label, setLabel] = useState('');
  const [project, setProject] = useState<string>('');
  const projects = useProjects().data ?? [];
  const d = date ?? next;

  const add = () => {
    switch (preset) {
      case 'imovel': {
        if (!value) return;
        const financed = Math.round(value * (1 - num(down) / 100));
        onAdd([
          {
            kind: 'one_off',
            label: t('planejamento-cenarios-compra-imovel'),
            date: d,
            amount: -value,
          },
          { kind: 'asset', label: t('planejamento-cenarios-imovel-bem'), date: d, value },
          ...(financed > 0
            ? [
                {
                  kind: 'loan' as const,
                  label: t('planejamento-cenarios-financiamento'),
                  date: d,
                  principal: financed,
                  annual_rate_pct: num(rate),
                  months: Math.max(Math.round(num(years) * 12), 1),
                },
              ]
            : []),
        ]);
        return;
      }
      case 'filho':
        if (!value) return;
        onAdd([
          {
            kind: 'monthly',
            label: t('planejamento-cenarios-filho-evento'),
            start: d,
            amount: -value,
          },
        ]);
        return;
      case 'pausa':
        onAdd([
          {
            kind: 'income_pct',
            label:
              num(pct) === 0
                ? t('planejamento-cenarios-pausa-evento')
                : t('planejamento-cenarios-renda-em', { pct: num(pct) }),
            start: d,
            months: Math.max(Math.round(num(months)), 1),
            pct: num(pct),
          },
        ]);
        return;
      case 'renda':
        if (!value) return;
        onAdd([
          {
            kind: 'monthly',
            label: label.trim() || t('planejamento-cenarios-mudanca-renda'),
            start: d,
            amount: value,
          },
        ]);
        return;
      case 'unico':
        if (!value) return;
        onAdd([
          {
            kind: 'one_off',
            label: label.trim() || t('planejamento-cenarios-gasto-unico'),
            date: d,
            amount: value,
          },
        ]);
        return;
      case 'projeto': {
        const p = projects.find((x) => x.id === project);
        if (!p) return;
        const left = Math.max((p.budget ?? 0) - p.spent, 0);
        if (!left) {
          toast({
            tone: 'info',
            title: t('planejamento-cenarios-sem-orcamento', { projeto: p.name }),
          });
          return;
        }
        onAdd([
          {
            kind: 'one_off',
            label: p.name,
            date: p.end_date && p.end_date > todayIso() ? p.end_date : d,
            amount: -left,
          },
        ]);
      }
    }
  };

  const when = (label2 = t('planejamento-cenarios-a-partir')) => (
    <Field label={label2}>
      <DateInput value={date} onValueChange={setDate} />
    </Field>
  );

  // Topo alinhado (não a base): a dica embaixo de um campo não sobe o rótulo e a caixa dele
  // em relação aos vizinhos. O botão desce a altura do rótulo para ficar na linha das caixas.
  return (
    <div className="grid items-start gap-3 @min-[720px]:grid-cols-4">
      {preset === 'imovel' ? (
        <>
          <Field label={t('planejamento-cenarios-preco-imovel')}>
            <MoneyInput value={value} onValueChange={setValue} />
          </Field>
          <Field label={t('planejamento-cenarios-entrada')}>
            <Input numeric value={down} onChange={(e) => setDown(e.target.value)} suffix="%" />
          </Field>
          <Field label={t('planejamento-cenarios-prazo')}>
            <Input
              numeric
              value={years}
              onChange={(e) => setYears(e.target.value)}
              suffix={t('planejamento-cenarios-anos')}
            />
          </Field>
          <Field
            label={t('planejamento-cenarios-juros-reais')}
            hint={t('planejamento-cenarios-juros-reais-dica')}
          >
            <Input
              numeric
              value={rate}
              onChange={(e) => setRate(e.target.value)}
              suffix={t('planejamento-cenarios-ao-ano')}
            />
          </Field>
          {when(t('planejamento-cenarios-data-compra'))}
        </>
      ) : null}
      {preset === 'filho' ? (
        <>
          <Field label={t('planejamento-cenarios-custo-mes')}>
            <MoneyInput value={value} onValueChange={setValue} />
          </Field>
          {when()}
        </>
      ) : null}
      {preset === 'pausa' ? (
        <>
          <Field
            label={t('planejamento-cenarios-renda-pausa')}
            hint={t('planejamento-cenarios-renda-pausa-dica')}
          >
            <Input numeric value={pct} onChange={(e) => setPct(e.target.value)} suffix="%" />
          </Field>
          <Field label={t('planejamento-cenarios-quanto-tempo')}>
            <Input
              numeric
              value={months}
              onChange={(e) => setMonths(e.target.value)}
              suffix={t('planejamento-cenarios-meses')}
            />
          </Field>
          {when()}
        </>
      ) : null}
      {preset === 'renda' || preset === 'unico' ? (
        <>
          <Field label={t('planejamento-nome')}>
            <Input
              value={label}
              onChange={(e) => setLabel(e.target.value)}
              placeholder={
                preset === 'renda'
                  ? t('planejamento-cenarios-novo-emprego')
                  : t('planejamento-cenarios-exemplo-unico')
              }
            />
          </Field>
          <Field
            label={
              preset === 'renda'
                ? t('planejamento-cenarios-diferenca-mes')
                : t('planejamento-valor')
            }
            hint={t('planejamento-cenarios-negativo-dica')}
          >
            <MoneyInput value={value} onValueChange={setValue} allowNegative />
          </Field>
          {when(preset === 'renda' ? t('planejamento-cenarios-a-partir') : t('planejamento-data'))}
        </>
      ) : null}
      {preset === 'projeto' ? (
        <>
          <Field
            label={t('planejamento-cenarios-projeto-campo')}
            hint={t('planejamento-cenarios-projeto-dica')}
          >
            <Select
              value={project}
              onValueChange={setProject}
              options={projects.map((p) => ({ value: p.id, label: p.name }))}
              placeholder={
                projects.length
                  ? t('planejamento-cenarios-escolha')
                  : t('planejamento-cenarios-nenhum-projeto')
              }
            />
          </Field>
          {when(t('planejamento-cenarios-sem-data-fim'))}
        </>
      ) : null}
      <div className="@min-[720px]:pt-[22px]">
        <Button variant="secondary" icon={<Icon name="plus" size={13} />} onClick={add}>
          {t('planejamento-cenarios-adicionar')}
        </Button>
      </div>
    </div>
  );
}

function Results({ r }: { r: ScenarioResult }) {
  const option = useCallback(
    (theme: ChartTheme) => {
      const base = baseOption(theme);
      // Um ponto por ano (o último mês de cada ano) para o gráfico ficar legível.
      const idx = r.base.points
        .map((_, i) => i)
        .filter((i) => (i + 1) % 12 === 0 || i === r.base.points.length - 1);
      const line = (name: string, data: number[], color: string, dashed = false) => ({
        name,
        type: 'line',
        symbol: 'none',
        color,
        lineStyle: { width: 1.75, ...(dashed ? { type: 'dashed' } : {}) },
        data,
      });
      return {
        ...base,
        legend: { ...(base.legend as object), show: true, top: 0 },
        grid: { left: 8, right: 16, top: 32, bottom: 8, containLabel: true },
        tooltip: {
          ...(base.tooltip as object),
          trigger: 'axis',
          valueFormatter: (v: number) => tooltipMoney(v, theme),
        },
        xAxis: categoryAxis(
          theme,
          idx.map((i) => formatMonthShort(r.base.points[i]!.month)),
        ),
        yAxis: valueAxis(theme, (v) => compactMoney(v, theme)),
        series: [
          line(
            t('planejamento-cenarios-patrimonio-base'),
            idx.map((i) => r.base.points[i]!.net_worth),
            theme.textDim,
            true,
          ),
          line(
            t('planejamento-cenarios-patrimonio-cenario'),
            idx.map((i) => r.scenario.points[i]!.net_worth),
            theme.series[0] ?? theme.accent,
          ),
          line(
            t('planejamento-cenarios-liquido-cenario'),
            idx.map((i) => r.scenario.points[i]!.liquid),
            theme.series[1] ?? theme.accent,
          ),
        ],
      };
    },
    [r],
  );
  const forecastOption = useCallback(
    (theme: ChartTheme) => {
      const base = baseOption(theme);
      const pts = r.forecast.points;
      return {
        ...base,
        legend: { ...(base.legend as object), show: true, top: 0 },
        grid: { left: 8, right: 16, top: 32, bottom: 8, containLabel: true },
        tooltip: {
          ...(base.tooltip as object),
          trigger: 'axis',
          valueFormatter: (v: number) => tooltipMoney(v, theme),
        },
        xAxis: (() => {
          const axis = categoryAxis(
            theme,
            pts.map((p) => formatDate(p.date).slice(0, 5)),
          );
          return {
            ...axis,
            boundaryGap: false,
            axisLabel: { ...(axis.axisLabel as object), hideOverlap: true },
          };
        })(),
        yAxis: valueAxis(theme, (v) => compactMoney(v, theme)),
        series: [
          {
            name: t('planejamento-cenarios-base'),
            type: 'line',
            step: 'end',
            symbol: 'none',
            color: theme.textDim,
            lineStyle: { type: 'dashed' },
            data: pts.map((p) => p.base),
          },
          {
            name: t('planejamento-cenarios-cenario'),
            type: 'line',
            step: 'end',
            symbol: 'none',
            color: theme.series[0] ?? theme.accent,
            data: pts.map((p) => p.scenario),
            markLine: {
              symbol: 'none',
              silent: true,
              lineStyle: { color: theme.negative, type: 'dotted' },
              label: { show: false },
              data: [{ yAxis: 0 }],
            },
          },
        ],
      };
    },
    [r],
  );
  const ret = r.retirement;
  return (
    <div className="flex flex-col gap-4">
      <div className="grid gap-3 @min-[720px]:grid-cols-3">
        <div className="flex flex-col gap-1 rounded-card border border-line bg-surface px-4 py-3">
          <span className="text-[12.5px] text-dim">
            {t('planejamento-cenarios-patrimonio-em', { anos: r.years })}
          </span>
          <Money cents={r.scenario.end_net_worth} className="text-[22px]" />
          <span className="text-[12px] text-dim">
            {tRich('planejamento-cenarios-base-valor', {
              valor: <Money cents={r.base.end_net_worth} />,
              diferenca: <Money cents={r.net_worth_delta} tone="auto" />,
            })}
          </span>
        </div>
        <div className="flex flex-col gap-1 rounded-card border border-line bg-surface px-4 py-3">
          <span className="text-[12.5px] text-dim">{t('planejamento-cenarios-liquido')}</span>
          <Money
            cents={r.scenario.end_liquid}
            className="text-[22px]"
            tone={r.scenario.end_liquid < 0 ? 'negative' : 'neutral'}
          />
          <span className="text-[12px] text-dim">
            {r.scenario.first_short && r.scenario.first_short === monthOf(todayIso())
              ? tRich('planejamento-cenarios-ja-negativos', {
                  diferenca: <Money cents={r.liquid_delta} tone="auto" />,
                })
              : r.scenario.first_short
                ? tRich('planejamento-cenarios-acabam', {
                    mes: formatMonthShort(r.scenario.first_short),
                    diferenca: <Money cents={r.liquid_delta} tone="auto" />,
                  })
                : tRich('planejamento-cenarios-nao-acabam', {
                    diferenca: <Money cents={r.liquid_delta} tone="auto" />,
                  })}
          </span>
        </div>
        <div className="flex flex-col gap-1 rounded-card border border-line bg-surface px-4 py-3">
          <span className="text-[12.5px] text-dim">{t('planejamento-cenarios-aposentadoria')}</span>
          {ret ? (
            <>
              <span className="num text-[22px]">
                {tRich(
                  'planejamento-cenarios-chance',
                  { pct: Math.round(ret.scenario_success * 100) },
                  (c) => (
                    <span className="text-[13px] text-dim">{c}</span>
                  ),
                )}
              </span>
              <span className="text-[12px] text-dim">
                {tRich('planejamento-cenarios-aposentadoria-base', {
                  pct: Math.round(ret.base_success * 100),
                  idade: ret.retirement_age,
                  valor: <Money cents={ret.scenario_wealth_at_retirement} />,
                })}
              </span>
            </>
          ) : (
            <span className="text-[12.5px] text-dim">{t('planejamento-cenarios-sem-idade')}</span>
          )}
        </div>
      </div>
      <p className="text-[13px] text-fg">{r.explanation}</p>
      <ChartFrame
        index="05.E1"
        title={t('planejamento-cenarios-grafico-patrimonio')}
        source={t('planejamento-cenarios-fonte-patrimonio')}
        chart={
          <EChart
            option={option}
            height={260}
            ariaLabel={t('planejamento-cenarios-grafico-patrimonio-aria')}
          />
        }
        table={
          <TechTable
            label={t('planejamento-cenarios-tabela-patrimonio')}
            prefsId="Patrimônio por mês"
            dense
            rows={r.scenario.points.filter((_, i) => (i + 1) % 12 === 0)}
            getRowId={(p) => p.month}
            columns={[
              {
                key: 'month',
                header: t('planejamento-cenarios-mes'),
                cell: (p) => formatMonthShort(p.month),
              },
              {
                key: 'base',
                header: t('planejamento-cenarios-base'),
                numeric: true,
                cell: (p) => (
                  <Money cents={r.base.points.find((b) => b.month === p.month)?.net_worth ?? 0} />
                ),
              },
              {
                key: 'net_worth',
                header: t('planejamento-cenarios-cenario'),
                numeric: true,
                cell: (p) => <Money cents={p.net_worth} />,
              },
              {
                key: 'liquid',
                header: t('planejamento-cenarios-liquido'),
                numeric: true,
                cell: (p) => <Money cents={p.liquid} />,
              },
            ]}
          />
        }
      />
      <ChartFrame
        index="05.E2"
        title={t('planejamento-cenarios-grafico-saldo')}
        source={
          r.forecast.scenario_first_negative
            ? t('planejamento-cenarios-saldo-negativo', {
                data: formatDate(r.forecast.scenario_first_negative),
              })
            : t('planejamento-cenarios-saldo-positivo')
        }
        chart={
          <EChart
            option={forecastOption}
            height={220}
            ariaLabel={t('planejamento-cenarios-grafico-saldo-aria')}
          />
        }
        table={
          <TechTable
            label={t('planejamento-cenarios-tabela-saldo')}
            prefsId="Previsão de saldo com o cenário"
            dense
            rows={r.forecast.points}
            getRowId={(p) => p.date}
            columns={[
              { key: 'date', header: t('planejamento-data'), cell: (p) => formatDate(p.date) },
              {
                key: 'base',
                header: t('planejamento-cenarios-base'),
                numeric: true,
                cell: (p) => <Money cents={p.base} />,
              },
              {
                key: 'scenario',
                header: t('planejamento-cenarios-cenario'),
                numeric: true,
                cell: (p) => <Money cents={p.scenario} />,
              },
            ]}
          />
        }
      />
      <ul className="flex flex-col gap-1 text-[12px] text-dim">
        {r.notes.map((n) => (
          <li key={n}>{n}</li>
        ))}
        <li>{t('planejamento-cenarios-aviso')}</li>
      </ul>
    </div>
  );
}

export function ScenariosPanel() {
  const [events, setEvents] = useState<ScenarioEvent[]>([]);
  const [preset, setPreset] = useState<Preset>('imovel');
  const [years, setYears] = useState<'5' | '10' | '20' | '30'>('10');
  const [name, setName] = useState('');
  const saved = useSavedScenarios().data ?? [];
  const save = useSaveScenario();
  const remove = useDeleteScenario();
  const args = useMemo(() => ({ events, years: Number(years) }), [events, years]);
  const q = useScenario(args, events.length > 0);
  // Recálculo terminado: a borda do card pisca (Fósforo). O recálculo já entra na atividade
  // pelo próprio comando (api/client.ts).
  const card = useSettleFlash<HTMLElement>(q.isFetching, q.isError);

  return (
    <InstrumentCard ref={card} index="05.E" title={t('planejamento-cenarios-titulo')}>
      <div className="flex flex-col gap-4">
        <p className="text-[13px] text-dim">{t('planejamento-cenarios-intro')}</p>
        {saved.length ? (
          <div className="flex flex-wrap items-center gap-2 text-[12.5px]">
            <span className="text-dim">{t('planejamento-cenarios-salvos')}</span>
            {saved.map((s) => (
              <span
                key={s.name}
                className="inline-flex items-center gap-1 rounded-control border border-line px-2 py-0.5"
              >
                <button
                  type="button"
                  className="hover:text-accent"
                  onClick={() => {
                    setEvents(s.events);
                    setName(s.name);
                  }}
                >
                  {s.name}
                </button>
                <IconButton
                  size="sm"
                  label={t('planejamento-cenarios-apagar', { nome: s.name })}
                  icon={<Icon name="x" size={11} />}
                  onClick={() => remove.mutate({ name: s.name })}
                />
              </span>
            ))}
          </div>
        ) : null}
        <Segmented<Preset>
          label={t('planejamento-cenarios-evento')}
          size="sm"
          value={preset}
          onChange={setPreset}
          options={presets()}
        />
        <PresetForm key={preset} preset={preset} onAdd={(e) => setEvents((xs) => [...xs, ...e])} />
        {events.length ? (
          <ul className="flex flex-col gap-1 rounded-card border border-line px-3 py-2 text-[13px]">
            {events.map((e, i) => (
              <li key={i} className="flex items-center gap-2">
                <span className="text-fg">{e.label}</span>
                <span className="min-w-0 flex-1 truncate text-dim">{describe(e)}</span>
                <IconButton
                  size="sm"
                  label={t('planejamento-tirar', { nome: e.label })}
                  icon={<Icon name="x" size={12} />}
                  onClick={() => setEvents((xs) => xs.filter((_, k) => k !== i))}
                />
              </li>
            ))}
          </ul>
        ) : null}
        {events.length ? (
          <div className="flex flex-wrap items-end gap-3">
            <Segmented
              label={t('planejamento-cenarios-horizonte')}
              size="sm"
              value={years}
              onChange={setYears}
              options={[
                { value: '5', label: t('planejamento-n-anos', { n: 5 }) },
                { value: '10', label: t('planejamento-n-anos', { n: 10 }) },
                { value: '20', label: t('planejamento-n-anos', { n: 20 }) },
                { value: '30', label: t('planejamento-n-anos', { n: 30 }) },
              ]}
            />
            <div className="ml-auto flex items-end gap-2">
              <Field label={t('planejamento-cenarios-nome')}>
                <Input
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder={t('planejamento-cenarios-nome-exemplo')}
                />
              </Field>
              <Button
                variant="ghost"
                disabled={!name.trim()}
                loading={save.isPending}
                onClick={() =>
                  save.mutate(
                    { name: name.trim(), events },
                    {
                      onSuccess: () =>
                        toast({ tone: 'success', title: t('planejamento-cenarios-salvo') }),
                    },
                  )
                }
              >
                {t('comum-salvar')}
              </Button>
              <Button variant="ghost" onClick={() => setEvents([])}>
                {t('planejamento-cenarios-limpar')}
              </Button>
            </div>
          </div>
        ) : null}
        <InlineError error={q.error ?? save.error} />
        {events.length && q.data ? <Results r={q.data} /> : null}
        <StudyNotice />
      </div>
    </InstrumentCard>
  );
}
