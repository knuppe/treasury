/**
 * "Para onde vai a renda" do mês: os quatro grupos de gasto (essenciais, dívidas, futuro, não
 * essencial) com valor, parte da renda, meta (editável) e uma barra contra a meta; a dica de
 * cada grupo e as maiores categorias ao abrir a linha.
 */
import { useState } from 'react';
import { Link } from 'react-router';

import type { Basis } from '@/api/generated/Basis';
import type { GroupTargets } from '@/api/generated/GroupTargets';
import type { SpendGroup } from '@/api/generated/SpendGroup';
import type { SpendGroupsReport } from '@/api/generated/SpendGroupsReport';
import { useSetGroupTargets } from '@/api/hooks/budget';
import { useSpendGroups } from '@/api/hooks/reports';
import { cssColor } from '@/charts/colorMath';
import {
  Button,
  Dialog,
  DialogClose,
  Field,
  Icon,
  InstrumentCard,
  Input,
  Money,
  Private,
  toast,
} from '@/design/components';
import { cx } from '@/design/cx';
import { QueryView } from '@/features/reports/controls';
import {
  DEFAULT_TARGETS,
  groupName,
  groupRows,
  groupTip,
  offTarget,
  SPEND_GROUPS,
  targetsSum,
  type GroupRow,
  type GroupStatus,
} from '@/features/reports/model/groups';
import { filtersSearch, parseFilters } from '@/features/reports/model/period';
import { useGroupColors } from '@/features/reports/useEntityColors';
import { InlineError } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatMonthLong, formatPercent, todayIso } from '@/lib/format';
import { monthBounds, monthOf, shiftMonth } from '@/lib/months';

/** Consulta dos grupos de um mês (a mesma do contexto da página, para o cache valer). */
export function useMonthGroups(month: string, basis: Basis, enabled = true) {
  return useSpendGroups({ from: month, to: month, basis }, enabled);
}

/** Relatório 12 · Grupos nos 12 meses até `month` (o atalho "12 meses" no mês atual). */
export function groupsReportLink(month: string): string {
  const today = todayIso();
  const base = parseFilters('', today);
  const current = month === monthOf(today);
  const search = filtersSearch(
    current
      ? { ...base, view: 'grupos', preset: '12m' }
      : {
          ...base,
          view: 'grupos',
          preset: 'personalizado',
          from: monthBounds(shiftMonth(month, -11)).from,
          to: monthBounds(month).to,
        },
  );
  return `/relatorios?${search}`;
}

const pct = (v: number) => formatPercent(v, { decimals: 0 });

export function StatusText({ status }: { status: GroupStatus }) {
  if (status === 'none') return null;
  const text = {
    over: t('orcamento-grupos-acima'),
    within: t('orcamento-grupos-dentro'),
    below: t('orcamento-grupos-abaixo'),
    reached: t('orcamento-grupos-atingida'),
  }[status];
  return (
    <span
      className={cx(
        'label-caps inline-flex items-center gap-1',
        offTarget(status) ? 'text-warning' : 'text-dim',
      )}
    >
      {offTarget(status) ? <Icon name="alert" size={11} /> : <Icon name="check" size={11} />}
      {text}
    </span>
  );
}

/** Barra da parte da renda (0 a 100%) com a marca da meta. */
function GroupBar({ row, color }: { row: GroupRow; color: string }) {
  const share = row.share ?? 0;
  const width = Math.max(0, Math.min(1, share)) * 100;
  return (
    <div
      role="img"
      aria-label={t('orcamento-grupos-barra', {
        grupo: groupName(row.group),
        pct: row.share == null ? '—' : pct(row.share),
      })}
      className="relative h-2.5 w-full overflow-visible rounded-[3px] bg-surface-2"
    >
      <div
        className="absolute inset-y-0 left-0 rounded-[3px]"
        style={{ width: `${width}%`, background: cssColor(color) }}
      />
      {row.target != null ? (
        <span
          aria-hidden
          className="absolute -inset-y-1 w-[2px] rounded-full bg-fg/70"
          style={{ left: `calc(${Math.min(1, row.target) * 100}% - 1px)` }}
        />
      ) : null}
    </div>
  );
}

function GroupLine({ row, color }: { row: GroupRow; color: string }) {
  const [open, setOpen] = useState(false);
  const name = groupName(row.group);
  return (
    <li className="border-b border-line/60 py-2.5 last:border-b-0">
      <div className="grid items-center gap-x-4 gap-y-2 @min-[790px]:grid-cols-[minmax(170px,240px)_minmax(0,1fr)_auto]">
        <div className="flex min-w-0 flex-col gap-0.5">
          <button
            type="button"
            aria-expanded={open}
            aria-label={
              open
                ? t('orcamento-grupos-esconder-maiores', { grupo: name })
                : t('orcamento-grupos-ver-maiores', { grupo: name })
            }
            onClick={() => setOpen((o) => !o)}
            className="flex min-w-0 items-center gap-2 rounded-control text-left text-[13px] font-medium text-fg hover:text-accent focus-visible:outline-2 focus-visible:outline-accent"
          >
            <Icon name={open ? 'chevronDown' : 'chevronRight'} size={12} className="text-dim" />
            <span
              aria-hidden
              className="size-2 shrink-0 rounded-[2px]"
              style={{ background: cssColor(color) }}
            />
            <span className="truncate">{name}</span>
          </button>
          <p className="pl-[34px] text-[12px] leading-snug text-dim">{groupTip(row.group)}</p>
        </div>
        <GroupBar row={row} color={color} />
        <div className="num flex flex-wrap items-baseline justify-end gap-x-3 gap-y-1 text-[12.5px]">
          <Money cents={row.amount} className="text-fg" />
          <span className="text-fg">
            <Private>
              {row.share == null ? '—' : t('orcamento-grupos-pct-renda', { pct: pct(row.share) })}
            </Private>
          </span>
          <span className="text-dim">
            {row.target == null
              ? t('orcamento-grupos-sem-meta')
              : t('orcamento-grupos-meta', { pct: pct(row.target) })}
          </span>
          <StatusText status={row.status} />
        </div>
      </div>
      {open ? (
        <div className="mt-2 pl-[34px]">
          {row.top.length ? (
            <ul
              aria-label={t('orcamento-grupos-maiores', { grupo: name })}
              className="flex flex-col gap-1 text-[12.5px]"
            >
              {row.top.map((c) => (
                <li
                  key={c.category_id ?? c.label}
                  className="flex max-w-[520px] items-baseline justify-between gap-4"
                >
                  <span className="truncate text-fg">{c.label}</span>
                  <Money cents={c.amount} className="text-dim" />
                </li>
              ))}
            </ul>
          ) : (
            <p className="text-[12.5px] text-dim">{t('orcamento-grupos-nada')}</p>
          )}
        </div>
      ) : null}
    </li>
  );
}

// ── Metas ────────────────────────────────────────────────────────

type Draft = Record<SpendGroup, string>;

const toDraft = (targets: GroupTargets): Draft =>
  Object.fromEntries(
    SPEND_GROUPS.map((g) => {
      const v = targets[g];
      return [g, v == null ? '' : String(Math.round(v * 1000) / 10)];
    }),
  ) as Draft;

/** Texto do campo → fração (vazio = sem meta, `undefined` = inválido). */
function parseTarget(text: string): number | null | undefined {
  const s = text.trim().replace('%', '').replace(',', '.');
  if (!s) return null;
  const n = Number(s);
  if (!Number.isFinite(n) || n < 0 || n > 100) return undefined;
  return Math.round(n * 10) / 1000;
}

function TargetsDialog({ targets }: { targets: GroupTargets }) {
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState<Draft>(() => toDraft(targets));
  const save = useSetGroupTargets();
  const parsed = Object.fromEntries(SPEND_GROUPS.map((g) => [g, parseTarget(draft[g])])) as Record<
    SpendGroup,
    number | null | undefined
  >;
  const invalid = SPEND_GROUPS.some((g) => parsed[g] === undefined);
  const next: GroupTargets = {
    essential: parsed.essential ?? null,
    debt: parsed.debt ?? null,
    future: parsed.future ?? null,
    non_essential: parsed.non_essential ?? null,
  };
  const sum = targetsSum(next);
  return (
    <Dialog
      open={open}
      onOpenChange={(o) => {
        setOpen(o);
        if (o) setDraft(toDraft(targets));
        else save.reset();
      }}
      size="sm"
      title={t('orcamento-grupos-metas-titulo')}
      description={t('orcamento-grupos-metas-descricao')}
      trigger={
        <Button size="sm" variant="ghost" icon={<Icon name="reticle" size={12} />}>
          {t('orcamento-grupos-editar-metas')}
        </Button>
      }
      footer={
        <>
          <Button variant="ghost" onClick={() => setDraft(toDraft(DEFAULT_TARGETS))}>
            {t('orcamento-grupos-padrao')}
          </Button>
          <DialogClose asChild>
            <Button variant="ghost">{t('comum-cancelar')}</Button>
          </DialogClose>
          <Button
            variant="primary"
            disabled={invalid}
            loading={save.isPending}
            onClick={() =>
              save.mutate(next, {
                onSuccess: () => {
                  setOpen(false);
                  toast({ tone: 'success', title: t('orcamento-grupos-metas-salvas') });
                },
              })
            }
          >
            {t('comum-salvar')}
          </Button>
        </>
      }
    >
      <div className="grid gap-3">
        {SPEND_GROUPS.map((g) => (
          <Field
            key={g}
            label={t('orcamento-grupos-meta-de', { grupo: groupName(g) })}
            error={parsed[g] === undefined ? t('orcamento-grupos-meta-invalida') : undefined}
          >
            <Input
              inputMode="decimal"
              value={draft[g]}
              onChange={(e) => setDraft((d) => ({ ...d, [g]: e.target.value }))}
              suffix="%"
            />
          </Field>
        ))}
      </div>
      <p className={cx('num text-[12.5px]', sum != null && sum > 1 ? 'text-warning' : 'text-dim')}>
        {t('orcamento-grupos-soma', { pct: sum == null ? '—' : pct(sum) })}
        {sum != null && sum > 1 ? ` ${t('orcamento-grupos-soma-acima')}` : null}
      </p>
      <InlineError error={save.error} />
    </Dialog>
  );
}

// ── Cartão ───────────────────────────────────────────────────────

function GroupsBody({ data, month }: { data: SpendGroupsReport; month: string }) {
  const colors = useGroupColors();
  const m = data.months.find((x) => x.month === month) ?? data.total;
  const rows = groupRows(m, data.targets);
  return (
    <div className="flex flex-col gap-3">
      {m.income <= 0 ? (
        <p role="note" className="text-[12.5px] text-warning">
          {t('orcamento-grupos-sem-renda')}
        </p>
      ) : null}
      <ul aria-label={t('orcamento-grupos-aria', { mes: formatMonthLong(month) })}>
        {rows.map((r) => (
          <GroupLine key={r.group} row={r} color={colors.get(r.group) ?? 'var(--line-strong)'} />
        ))}
      </ul>
      <div className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
        <p className="text-[12px] text-dim">{t('orcamento-grupos-futuro-nota')}</p>
        <Link
          to={groupsReportLink(month)}
          className="label-caps inline-flex items-center gap-1 hover:text-accent"
        >
          {t('orcamento-grupos-relatorios')} <Icon name="arrowRight" size={11} />
        </Link>
      </div>
    </div>
  );
}

export function SpendGroupsCard({
  month,
  basis,
  index,
}: {
  month: string;
  basis: Basis;
  index: string;
}) {
  const query = useMonthGroups(month, basis);
  const m = query.data?.months.find((x) => x.month === month);
  return (
    <InstrumentCard
      index={index}
      title={t('orcamento-grupos-titulo')}
      actions={query.data ? <TargetsDialog targets={query.data.targets} /> : undefined}
      footer={
        m
          ? [
              { label: t('orcamento-grupos-renda'), value: <Money cents={m.income} /> },
              { label: t('orcamento-grupos-total'), value: <Money cents={m.spending} /> },
              {
                label: t('orcamento-grupos-livre'),
                value: (
                  <Money
                    cents={m.income - m.spending}
                    tone={m.income - m.spending < 0 ? 'negative' : 'neutral'}
                  />
                ),
              },
              {
                label: t('casca-tela-ajustes'),
                value: (
                  <Link to="/ajustes?secao=categorias" className="hover:text-accent">
                    {t('orcamento-grupos-categorias')}
                  </Link>
                ),
              },
            ]
          : undefined
      }
    >
      <QueryView query={query} loadingLabel={t('orcamento-grupos-carregando')}>
        {(data) => <GroupsBody data={data} month={month} />}
      </QueryView>
    </InstrumentCard>
  );
}
