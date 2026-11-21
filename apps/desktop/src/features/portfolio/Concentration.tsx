/**
 * Concentração da carteira (docs/concentration.md): o card com os ativos, grupos e classes acima
 * do limite (peso de hoje, limite e quanto passa, em dinheiro), a etiqueta colorida do peso na
 * tabela de posições e o diálogo dos limites: padrões de ativo e de grupo, o limite próprio de
 * cada ativo e as etiquetas (quais ativos têm cada uma e quais somam como grupo).
 *
 * É leitura dos dados, sem recomendação de compra ou venda (CVM Res. 20/2021): os limites são da
 * família, e o card só diz onde o peso passou deles.
 */
import { useMemo, useState } from 'react';

import type { AssetLabel } from '@/api/generated/AssetLabel';
import type { ConcentrationAlert } from '@/api/generated/ConcentrationAlert';
import type { ConcentrationKind } from '@/api/generated/ConcentrationKind';
import type { ConcentrationReport } from '@/api/generated/ConcentrationReport';
import { useAssetLabels, useSetAssetLabels, useUpdateAssetLabel } from '@/api/hooks/assets';
import { useConcentration, useSetConcentrationLimit } from '@/api/hooks/portfolio';
import {
  Button,
  ChipsInput,
  Dialog,
  DialogClose,
  Icon,
  InstrumentCard,
  Money,
  toast,
  Tooltip,
} from '@/design/components';
import { cx } from '@/design/cx';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatPercent } from '@/lib/format';
import { FormError } from '@/vault/FormError';

import { DecimalInput } from './components/DecimalInput';
import { Pct } from './components/Pct';
import { ErrorBlock, LoadingBlock } from './components/States';
import { formatDecimal } from './decimal';

const KIND_LABEL: Record<ConcentrationKind, () => string> = {
  asset: () => t('carteira-conc-tipo-ativo'),
  group: () => t('carteira-conc-tipo-grupo'),
  class: () => t('carteira-conc-tipo-classe'),
};

const limitText = (limit: number) => formatPercent(limit, { decimals: limit * 100 >= 10 ? 0 : 1 });

const TONE = {
  warning: {
    text: 'text-warning',
    edge: 'border-l-warning',
    pill: 'border-warning/50 bg-[var(--warning-soft)] text-warning',
  },
  critical: {
    text: 'text-negative',
    edge: 'border-l-negative',
    pill: 'border-negative/50 bg-[var(--negative-soft)] text-negative',
  },
} as const;

/** Peso na tabela de posições quando o ativo passou do limite (laranja; vermelho a 1,5×). */
export function WeightPill({
  alert,
  weight,
}: {
  alert: ConcentrationAlert;
  weight: number | null;
}) {
  const hint = t('carteira-conc-pill-dica', { limite: limitText(alert.limit) });
  return (
    <Tooltip content={hint}>
      <button
        type="button"
        className={cx(
          'inline-flex items-center rounded-full border px-1.5 py-px',
          TONE[alert.severity].pill,
        )}
      >
        <Pct value={weight} />
        <span className="sr-only">{`: ${hint}`}</span>
      </button>
    </Tooltip>
  );
}

function AlertRow({ alert }: { alert: ConcentrationAlert }) {
  const tone = TONE[alert.severity];
  const members =
    alert.kind === 'group' ? alert.assets.map((a) => a.ticker ?? a.name).join(', ') : null;
  return (
    <li
      className={cx(
        'flex flex-wrap items-baseline gap-x-4 gap-y-1 rounded-control border border-l-2 border-line px-3 py-2',
        tone.edge,
      )}
    >
      <span className="flex min-w-0 flex-1 items-baseline gap-2">
        <span className="truncate text-[13px] font-medium text-fg">{alert.label}</span>
        <span className="label-caps">{KIND_LABEL[alert.kind]()}</span>
        {members ? <span className="truncate text-[12px] text-dim">{members}</span> : null}
      </span>
      <span className="flex items-baseline gap-1.5 text-[12.5px]">
        <Pct value={alert.weight} className={tone.text} />
        <span className="text-dim">
          {t('carteira-conc-limite', { limite: limitText(alert.limit) })}
        </span>
      </span>
      <span className="text-[12.5px] text-fg">
        {withSlots(t('carteira-conc-acima', { valor: slot(0) }), [
          <Money key="v" cents={alert.excess_base} />,
        ])}
      </span>
      <span className={cx('label-caps', tone.text)}>
        {alert.severity === 'critical'
          ? t('carteira-conc-sev-critical')
          : t('carteira-conc-sev-warning')}
      </span>
    </li>
  );
}

/** Card do topo das posições: um alerta por linha, os críticos primeiro (a ordem do núcleo). */
export function ConcentrationCard({
  alerts,
  onAdjust,
}: {
  alerts: readonly ConcentrationAlert[];
  onAdjust: () => void;
}) {
  return (
    <InstrumentCard
      title={t('carteira-conc-titulo')}
      headingLevel={3}
      actions={
        <Button
          size="sm"
          variant="ghost"
          icon={<Icon name="sliders" size={12} />}
          onClick={onAdjust}
        >
          {t('carteira-conc-ajustar')}
        </Button>
      }
      footer={<p className="text-[12px] text-dim">{t('carteira-conc-rodape')}</p>}
    >
      <ul aria-label={t('carteira-conc-lista')} className="flex flex-col gap-1.5">
        {alerts.map((a) => (
          <AlertRow key={`${a.kind}:${a.ref}`} alert={a} />
        ))}
      </ul>
    </InstrumentCard>
  );
}

// ── Diálogo dos limites ──────────────────────────────────────────

/** % do núcleo (20, 12,5) como decimal canônico do campo. */
const pctDecimal = (pct: number) => String(Math.round(pct * 100) / 100);

const validPct = (v: string | null) => v == null || (Number(v) > 0 && Number(v) <= 100);

const sameList = (a: readonly string[], b: readonly string[]) =>
  a.length === b.length && a.every((x, i) => x === b[i]);

interface LabelDraft {
  group?: boolean;
  limit?: string | null;
}

/** Rascunho do diálogo: só o que a pessoa mexeu (`undefined` = como está). */
interface Draft {
  assetDefault?: string | null;
  groupDefault?: string | null;
  limits: Record<string, string | null>;
  labels: Record<string, string[]>;
  groups: Record<string, LabelDraft>;
}

const EMPTY_DRAFT: Draft = { limits: {}, labels: {}, groups: {} };

function originalLabels(labels: readonly AssetLabel[], assetId: string): string[] {
  return labels.filter((l) => l.asset_ids.includes(assetId)).map((l) => l.name);
}

const ROW_COLS = 'grid grid-cols-[minmax(0,0.8fr)_64px_92px_minmax(0,2fr)] items-center gap-2';

/**
 * Limites de concentração e etiquetas. Tudo fica em rascunho até "Salvar limites"; cada mudança
 * vira uma escrita do núcleo (os limites, as etiquetas de cada ativo e o papel de grupo).
 */
export function ConcentrationLimitsDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const reportQ = useConcentration({}, { enabled: open });
  const labelsQ = useAssetLabels({ enabled: open });
  const setLimit = useSetConcentrationLimit();
  const setLabels = useSetAssetLabels();
  const updateLabel = useUpdateAssetLabel();
  const [draft, setDraft] = useState<Draft>(EMPTY_DRAFT);
  const [error, setError] = useState<Error | null>(null);
  const [saving, setSaving] = useState(false);
  // Cada abertura começa do que está gravado.
  const [wasOpen, setWasOpen] = useState(open);
  if (open !== wasOpen) {
    setWasOpen(open);
    if (open) {
      setDraft(EMPTY_DRAFT);
      setError(null);
    }
  }

  const report = reportQ.data;
  const labels = useMemo(() => labelsQ.data ?? [], [labelsQ.data]);
  const assetDefault =
    draft.assetDefault !== undefined
      ? draft.assetDefault
      : report
        ? pctDecimal(report.asset_default_pct)
        : null;
  const groupDefault =
    draft.groupDefault !== undefined
      ? draft.groupDefault
      : report
        ? pctDecimal(report.group_default_pct)
        : null;
  const limitOf = (id: string): string | null => {
    if (id in draft.limits) return draft.limits[id] ?? null;
    const item = report?.assets.find((a) => a.ref === id);
    return item?.custom_limit ? pctDecimal(item.limit * 100) : null;
  };
  const labelsOf = (id: string) => draft.labels[id] ?? originalLabels(labels, id);

  const invalid =
    !validPct(assetDefault) ||
    !validPct(groupDefault) ||
    Object.values(draft.limits).some((v) => !validPct(v)) ||
    Object.values(draft.groups).some((g) => g.limit !== undefined && !validPct(g.limit));

  const save = async () => {
    if (!report) return;
    setError(null);
    setSaving(true);
    try {
      if (
        draft.assetDefault !== undefined &&
        draft.assetDefault !== pctDecimal(report.asset_default_pct)
      )
        await setLimit.mutateAsync({
          scope: 'asset_default',
          pct: draft.assetDefault == null ? null : Number(draft.assetDefault),
        });
      if (
        draft.groupDefault !== undefined &&
        draft.groupDefault !== pctDecimal(report.group_default_pct)
      )
        await setLimit.mutateAsync({
          scope: 'group_default',
          pct: draft.groupDefault == null ? null : Number(draft.groupDefault),
        });
      for (const [id, v] of Object.entries(draft.limits)) {
        const item = report.assets.find((a) => a.ref === id);
        const before = item?.custom_limit ? pctDecimal(item.limit * 100) : null;
        if (v !== before)
          await setLimit.mutateAsync({
            scope: 'asset',
            asset_id: id,
            pct: v == null ? null : Number(v),
          });
      }
      for (const [id, names] of Object.entries(draft.labels))
        if (!sameList(names, originalLabels(labels, id)))
          await setLabels.mutateAsync({ asset_id: id, labels: names });
      for (const [name, g] of Object.entries(draft.groups)) {
        const before = labels.find((l) => l.name === name);
        const group =
          g.group !== undefined && g.group !== (before?.is_concentration_group ?? false);
        const beforeLimit =
          before?.concentration_limit_pct != null
            ? pctDecimal(before.concentration_limit_pct)
            : null;
        const limit = g.limit !== undefined && g.limit !== beforeLimit;
        if (group || limit)
          await updateLabel.mutateAsync({
            name,
            ...(group ? { is_concentration_group: g.group } : {}),
            ...(limit ? { concentration_limit_pct: g.limit == null ? null : Number(g.limit) } : {}),
          });
      }
      toast({ tone: 'success', title: t('carteira-conc-salvos') });
      onOpenChange(false);
    } catch (e) {
      setError(e instanceof Error ? e : new Error(String(e)));
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="lg"
      title={t('carteira-conc-dialogo-titulo')}
      description={t('carteira-conc-dialogo-texto')}
      footer={
        <>
          <DialogClose asChild>
            <Button variant="ghost">{t('comum-cancelar')}</Button>
          </DialogClose>
          <Button
            variant="primary"
            disabled={!report || invalid}
            loading={saving}
            onClick={() => void save()}
          >
            {t('carteira-conc-salvar')}
          </Button>
        </>
      }
    >
      {reportQ.isPending || labelsQ.isPending ? (
        <LoadingBlock />
      ) : reportQ.isError ? (
        <ErrorBlock error={reportQ.error} onRetry={() => void reportQ.refetch()} />
      ) : labelsQ.isError ? (
        <ErrorBlock error={labelsQ.error} onRetry={() => void labelsQ.refetch()} />
      ) : report ? (
        <div className="flex flex-col gap-5">
          <Defaults
            assetDefault={assetDefault}
            groupDefault={groupDefault}
            onAsset={(v) => setDraft({ ...draft, assetDefault: v })}
            onGroup={(v) => setDraft({ ...draft, groupDefault: v })}
          />
          <AssetRows
            report={report}
            labels={labels}
            assetDefault={assetDefault}
            limitOf={limitOf}
            labelsOf={labelsOf}
            onLimit={(id, v) => setDraft({ ...draft, limits: { ...draft.limits, [id]: v } })}
            onLabels={(id, v) => setDraft({ ...draft, labels: { ...draft.labels, [id]: v } })}
          />
          <LabelRows
            report={report}
            labels={labels}
            draft={draft}
            labelsOf={labelsOf}
            groupDefault={groupDefault}
            onChange={(name, patch) =>
              setDraft({
                ...draft,
                groups: { ...draft.groups, [name]: { ...draft.groups[name], ...patch } },
              })
            }
          />
          {invalid ? (
            <p role="alert" className="text-[12.5px] text-negative">
              {t('carteira-conc-limite-invalido')}
            </p>
          ) : null}
          <FormError error={error} />
        </div>
      ) : null}
    </Dialog>
  );
}

function Defaults({
  assetDefault,
  groupDefault,
  onAsset,
  onGroup,
}: {
  assetDefault: string | null;
  groupDefault: string | null;
  onAsset: (v: string | null) => void;
  onGroup: (v: string | null) => void;
}) {
  return (
    <section aria-label={t('carteira-conc-padroes')} className="flex flex-col gap-2">
      <h3 className="label-caps text-fg">{t('carteira-conc-padroes')}</h3>
      <div className="flex flex-wrap gap-4">
        <label className="flex flex-col gap-1 text-[12.5px] text-dim">
          {t('carteira-conc-padrao-ativo')}
          <DecimalInput
            value={assetDefault}
            maxDecimals={2}
            onValueChange={onAsset}
            placeholder="20"
            wrapperClassName="h-8 w-28"
          />
        </label>
        <label className="flex flex-col gap-1 text-[12.5px] text-dim">
          {t('carteira-conc-padrao-grupo')}
          <DecimalInput
            value={groupDefault}
            maxDecimals={2}
            onValueChange={onGroup}
            placeholder="30"
            wrapperClassName="h-8 w-28"
          />
        </label>
      </div>
    </section>
  );
}

function AssetRows({
  report,
  labels,
  assetDefault,
  limitOf,
  labelsOf,
  onLimit,
  onLabels,
}: {
  report: ConcentrationReport;
  labels: readonly AssetLabel[];
  assetDefault: string | null;
  limitOf: (id: string) => string | null;
  labelsOf: (id: string) => string[];
  onLimit: (id: string, v: string | null) => void;
  onLabels: (id: string, v: string[]) => void;
}) {
  const suggestions = useMemo(() => labels.map((l) => l.name), [labels]);
  return (
    <section aria-label={t('carteira-conc-por-ativo')} className="flex flex-col gap-2">
      <h3 className="label-caps text-fg">{t('carteira-conc-por-ativo')}</h3>
      {report.assets.length === 0 ? (
        <p className="text-[12.5px] text-dim">{t('carteira-conc-sem-ativos')}</p>
      ) : (
        <div role="table" aria-label={t('carteira-conc-por-ativo')} className="flex flex-col">
          <div role="row" className={cx(ROW_COLS, 'label-caps border-b border-line pb-1.5')}>
            <span role="columnheader">{t('carteira-col-ativo')}</span>
            <span role="columnheader" className="text-right">
              {t('carteira-col-peso')}
            </span>
            <span role="columnheader" className="pr-3 text-right">
              {t('carteira-conc-col-limite')}
            </span>
            <span role="columnheader">{t('carteira-etiqueta-titulo')}</span>
          </div>
          {report.assets.map((a) => (
            <div role="row" key={a.ref} className={cx(ROW_COLS, 'border-b border-line/60 py-1.5')}>
              <span role="cell" className="truncate text-[13px] text-fg" title={a.assets[0]?.name}>
                {a.label}
              </span>
              <span role="cell" className="text-right text-[12.5px]">
                <Pct
                  value={a.weight}
                  className={a.weight > a.limit ? 'text-warning' : 'text-dim'}
                />
              </span>
              <span role="cell">
                <DecimalInput
                  aria-label={t('carteira-conc-limite-de', { ativo: a.label })}
                  value={limitOf(a.ref)}
                  maxDecimals={2}
                  onValueChange={(v) => onLimit(a.ref, v)}
                  placeholder={
                    a.exempt
                      ? t('carteira-conc-isento')
                      : assetDefault
                        ? formatDecimal(assetDefault, { maxDecimals: 2 })
                        : '—'
                  }
                  title={a.exempt ? t('carteira-conc-isento-dica') : undefined}
                  wrapperClassName="h-8"
                />
              </span>
              <span role="cell" className="min-w-0">
                <ChipsInput
                  value={labelsOf(a.ref)}
                  onChange={(v) => onLabels(a.ref, v)}
                  label={t('carteira-etiqueta-de', { ativo: a.label })}
                  removeLabel={(x) => t('carteira-etiqueta-tirar', { etiqueta: x, ativo: a.label })}
                  suggestions={suggestions}
                  placeholder={t('carteira-etiqueta-exemplo')}
                />
              </span>
            </div>
          ))}
        </div>
      )}
      <p className="text-[12px] text-dim">{t('carteira-conc-por-ativo-texto')}</p>
    </section>
  );
}

function LabelRows({
  report,
  labels,
  draft,
  labelsOf,
  groupDefault,
  onChange,
}: {
  report: ConcentrationReport;
  labels: readonly AssetLabel[];
  draft: Draft;
  labelsOf: (id: string) => string[];
  groupDefault: string | null;
  onChange: (name: string, patch: LabelDraft) => void;
}) {
  // As etiquetas gravadas e as novas do rascunho, com os ativos de cada uma como estão na tela.
  const held = new Map(report.assets.map((a) => [a.ref, a.label]));
  const names = new Map<string, string>();
  for (const l of labels) names.set(l.name.toLowerCase(), l.name);
  for (const a of report.assets)
    for (const n of labelsOf(a.ref)) if (!names.has(n.toLowerCase())) names.set(n.toLowerCase(), n);
  const rows = [...names.values()]
    .sort((a, b) => a.localeCompare(b, undefined, { sensitivity: 'base' }))
    .map((name) => {
      const saved = labels.find((l) => l.name === name);
      const members = report.assets
        .filter((a) => labelsOf(a.ref).some((n) => n.toLowerCase() === name.toLowerCase()))
        .map((a) => a.label);
      const others = (saved?.asset_ids ?? []).filter((id) => !held.has(id)).length;
      const d = draft.groups[name] ?? {};
      const group = d.group ?? saved?.is_concentration_group ?? false;
      const limit =
        d.limit !== undefined
          ? d.limit
          : saved?.concentration_limit_pct != null
            ? pctDecimal(saved.concentration_limit_pct)
            : null;
      return { name, members, others, group, limit };
    });
  return (
    <section aria-label={t('carteira-etiqueta-titulo')} className="flex flex-col gap-2">
      <h3 className="label-caps text-fg">{t('carteira-etiqueta-titulo')}</h3>
      <p className="text-[12px] text-dim">{t('carteira-etiqueta-texto')}</p>
      {rows.length === 0 ? (
        <p className="text-[12.5px] text-dim">{t('carteira-etiqueta-sem')}</p>
      ) : (
        <ul className="flex flex-col">
          {rows.map((r) => (
            <li
              key={r.name}
              className="flex flex-wrap items-center gap-x-4 gap-y-1.5 border-b border-line/60 py-2"
            >
              <span className="flex min-w-0 flex-1 flex-col leading-tight">
                <span className="text-[13px] font-medium text-fg">{r.name}</span>
                <span className="truncate text-[12px] text-dim">
                  {r.members.length || r.others
                    ? [
                        ...r.members,
                        ...(r.others ? [t('carteira-etiqueta-outros', { n: r.others })] : []),
                      ].join(', ')
                    : t('carteira-etiqueta-sem-ativos')}
                </span>
              </span>
              <label className="flex items-center gap-2 text-[12.5px] text-fg">
                <input
                  type="checkbox"
                  className="size-3.5"
                  aria-label={t('carteira-etiqueta-grupo-de', { etiqueta: r.name })}
                  checked={r.group}
                  onChange={(e) => onChange(r.name, { group: e.target.checked })}
                />
                {t('carteira-etiqueta-grupo')}
              </label>
              {r.group ? (
                <span className="w-24">
                  <DecimalInput
                    aria-label={t('carteira-etiqueta-limite-de', { etiqueta: r.name })}
                    value={r.limit}
                    maxDecimals={2}
                    onValueChange={(v) => onChange(r.name, { limit: v })}
                    placeholder={
                      groupDefault ? formatDecimal(groupDefault, { maxDecimals: 2 }) : '—'
                    }
                    wrapperClassName="h-8"
                  />
                </span>
              ) : null}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
