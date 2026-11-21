/**
 * Screener (`research.screen`): filtros `campo op valor` com E/OU e grupos, recorte do
 * universo, colunas escolhidas e telas salvas (`research.screens.*`). As telas são dados do
 * usuário: gravadas pelo núcleo com histórico, com "Desfazer" logo depois de salvar ou
 * excluir (e em Ajustes › Histórico de alterações).
 */
import { useEffect, useMemo, useRef, useState, type FormEvent } from 'react';
import { useNavigate } from 'react-router';

import { call } from '@/api/client';
import type { ChangeSetSummary } from '@/api/generated/ChangeSetSummary';
import type { IndicatorInfo } from '@/api/generated/IndicatorInfo';
import type { Screen } from '@/api/generated/Screen';
import type { ScreenArgs } from '@/api/generated/ScreenArgs';
import type { ScreenRow } from '@/api/generated/ScreenRow';
import type { SortOrder } from '@/api/generated/SortOrder';
import { useUndo } from '@/api/hooks/changes';
import {
  useCatalog,
  useCreateScreen,
  useDeleteScreen,
  useResearchStatus,
  useScreen,
  useScreens,
  useMethodBadges,
  useMethods,
  useTags,
  useUpdateScreen,
} from '@/api/hooks/research';
import {
  Button,
  Dialog,
  DialogClose,
  Field,
  Icon,
  Input,
  InstrumentCard,
  Select,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { cx } from '@/design/cx';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { FormError } from '@/vault/FormError';

import { Check } from '../portfolio/components/Fields';
import { Segmented } from '../portfolio/components/Segmented';
import { ConfirmButton, ErrorBlock, LoadingBlock, Notice } from '../portfolio/components/States';
import { CLASS_LABEL, safeDate } from '../portfolio/labels';
import { NoUniverse, subjectHref, TickerLink } from './common';
import { ConditionBuilder } from './ConditionBuilder';
import { formatIndicatorValue } from './format';
import { GROUP_LABEL, PANEL_GROUPS } from './labels';
import { TagLabels, TagPicker, tagsFor } from './TagPicker';
import {
  filterFields,
  fromScreenQuery,
  newCondition,
  toScreenQuery,
  type DraftQuery,
} from './query';
import { UniverseFilters } from './UniverseFilters';

const DEFAULT_COLUMNS = ['price', 'market_cap', 'pl', 'pvp', 'dy', 'roe', 'liquidity'];

function exampleDraft(): DraftQuery {
  const pl = newCondition('pl', 'lt');
  const dy = newCondition('dy', 'gte');
  return {
    logic: 'and',
    conditions: [
      { ...pl, value: '12' },
      { ...dy, value: '6' },
    ],
    groups: [],
    universe: {},
  };
}

/**
 * Recorte que o Panorama do Mercado abre (duplo clique num setor ou segmento): a classe certa
 * (ações do setor, FIIs do segmento) e só os papéis robustos e saudáveis pelo score.
 */
function sliceDraft(sector: string | null, segment: string | null): DraftQuery {
  const band = newCondition('score_band', 'in');
  return {
    logic: 'and',
    conditions: [{ ...band, value: 'robusto, saudavel' }],
    groups: [],
    universe: sector
      ? { classes: ['stock_br'], sectors: [sector] }
      : { classes: ['fii'], segments: [segment ?? ''] },
  };
}

/** Colunas do recorte: o score primeiro; FII sem P/L e ROE. */
const SLICE_COLUMNS = {
  stocks: ['price', 'score', 'market_cap', 'pl', 'pvp', 'dy', 'roe', 'liquidity'],
  fiis: ['price', 'score', 'pvp', 'dy', 'liquidity'],
};

/** Selos dos métodos que o papel cumpre hoje (nome curto, receita na aba Métodos). */
function MethodBadges({ keys, names }: { keys: string[]; names: Record<string, string> }) {
  if (!keys.length) return <span className="text-dim">—</span>;
  return (
    <span className="flex flex-wrap gap-1">
      {keys.map((k) => (
        <span
          key={k}
          className="rounded-control border border-line px-1 font-sans text-[10.5px] text-fg"
        >
          {names[k] ?? k}
        </span>
      ))}
    </span>
  );
}

export function resultColumns(
  chosen: readonly string[],
  filters: string[],
  sortBy: string,
): string[] {
  return [...new Set([...chosen, ...filters, sortBy])];
}

function metricColumns(keys: string[], catalog: readonly IndicatorInfo[]): TechColumn<ScreenRow>[] {
  return keys.flatMap((k) => {
    const info = catalog.find((i) => i.key === k);
    if (!info) return [];
    return [
      {
        key: k,
        header: info.label,
        numeric: true,
        cell: (r: ScreenRow) => formatIndicatorValue(r.metrics[k], info.unit, r.currency),
        sortValue: (r: ScreenRow) => r.metrics[k] ?? null,
      },
    ];
  });
}

async function lastChangeOf(rowId: string): Promise<string | null> {
  const list = await call<ChangeSetSummary[]>('changes.list', {
    table_name: 'screen',
    row_id: rowId,
    limit: 1,
  });
  return list[0]?.id ?? null;
}

function SaveDialog({
  open,
  onOpenChange,
  onSave,
  pending,
  error,
}: {
  open: boolean;
  onOpenChange: (o: boolean) => void;
  onSave: (name: string, description: string) => void;
  pending: boolean;
  error: Error | null;
}) {
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [submitted, setSubmitted] = useState(false);
  const submit = (e: FormEvent) => {
    e.preventDefault();
    setSubmitted(true);
    if (name.trim()) onSave(name.trim(), description.trim());
  };
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="sm"
      index="07"
      title={t('pesquisa-scr-salvar-titulo')}
      description={t('pesquisa-scr-salvar-texto')}
    >
      <form
        onSubmit={submit}
        className="flex flex-col gap-4"
        aria-label={t('pesquisa-scr-salvar-rotulo')}
        noValidate
      >
        <Field
          label={t('pesquisa-scr-nome')}
          error={submitted && !name.trim() ? t('pesquisa-scr-informe-nome') : undefined}
        >
          <Input value={name} onChange={(e) => setName(e.target.value)} autoFocus />
        </Field>
        <Field label={t('pesquisa-scr-descricao')}>
          <Input value={description} onChange={(e) => setDescription(e.target.value)} />
        </Field>
        <FormError error={error} />
        <div className="flex justify-end gap-2 border-t border-line pt-3">
          <DialogClose asChild>
            <Button variant="ghost">{t('comum-cancelar')}</Button>
          </DialogClose>
          <Button type="submit" variant="primary" loading={pending}>
            {t('pesquisa-scr-salvar')}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}

function ColumnsPicker({
  catalog,
  value,
  onChange,
}: {
  catalog: readonly IndicatorInfo[];
  value: string[];
  onChange: (v: string[]) => void;
}) {
  const groups = [...PANEL_GROUPS, 'fair_price' as const]
    .map((g) => ({ g, items: catalog.filter((i) => i.group === g) }))
    .filter((x) => x.items.length);
  return (
    <details className="rounded-card border border-line bg-surface px-3 py-2">
      <summary className="label-caps cursor-pointer text-fg">
        {t('pesquisa-scr-colunas', { n: value.length })}
      </summary>
      <div className="mt-3 grid gap-4 @min-[400px]:grid-cols-2 @min-[1040px]:grid-cols-4">
        {groups.map(({ g, items }) => (
          <fieldset key={g} className="flex flex-col gap-1.5">
            <legend className="label-caps mb-1">{GROUP_LABEL[g]}</legend>
            {items.map((i) => (
              <Check
                key={i.key}
                label={i.label}
                checked={value.includes(i.key)}
                onChange={(on) =>
                  onChange(on ? [...value, i.key] : value.filter((k) => k !== i.key))
                }
              />
            ))}
          </fieldset>
        ))}
      </div>
    </details>
  );
}

/** `sector`/`segment`: começa com o recorte do setor (o Panorama do Mercado abre assim). */
export function Screener({
  sector = null,
  segment = null,
}: { sector?: string | null; segment?: string | null } = {}) {
  const navigate = useNavigate();
  const catalogQ = useCatalog();
  const catalog = useMemo(() => catalogQ.data ?? [], [catalogQ.data]);
  const screensQ = useScreens();
  const status = useResearchStatus();
  const create = useCreateScreen();
  const update = useUpdateScreen();
  const badges = useMethodBadges();
  const methodsQ = useMethods();
  const methodNames = useMemo(
    () => Object.fromEntries((methodsQ.data ?? []).map((m) => [m.key, m.name])),
    [methodsQ.data],
  );
  const remove = useDeleteScreen();
  const undo = useUndo();

  const slice = sector != null || segment != null;
  const [draft, setDraft] = useState<DraftQuery>(() =>
    slice ? sliceDraft(sector, segment) : exampleDraft(),
  );
  const [draftKey, setDraftKey] = useState(0);
  const [columns, setColumns] = useState<string[]>(() =>
    !slice ? DEFAULT_COLUMNS : sector ? SLICE_COLUMNS.stocks : SLICE_COLUMNS.fiis,
  );
  const [sortBy, setSortBy] = useState(slice ? 'score' : 'market_cap');
  const [order, setOrder] = useState<'natural' | SortOrder>('natural');
  const [args, setArgs] = useState<ScreenArgs | null>(null);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [currentId, setCurrentId] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [lastChange, setLastChange] = useState<{ id: string; message: string } | null>(null);

  const screens = screensQ.data ?? [];
  const current = screens.find((s) => s.id === currentId) ?? null;
  const result = useScreen(args);
  const [tags, setTags] = useState<string[]>([]);
  const tagsQ = useTags();
  const allTags = useMemo(() => tagsQ.data ?? [], [tagsQ.data]);
  const parsed = useMemo(() => toScreenQuery(draft, catalog), [draft, catalog]);
  // As características marcadas entram no filtro (o papel precisa ter todas).
  const built = useMemo(
    () => ({
      ...parsed,
      query: parsed.query && tags.length ? { ...parsed.query, tags } : parsed.query,
    }),
    [parsed, tags],
  );
  const shown = resultColumns(columns, built.query ? filterFields(built.query) : [], sortBy);

  const run = () => {
    setErrors(built.errors);
    if (!built.query) return;
    setArgs({
      filters: built.query,
      sort_by: sortBy,
      ...(order === 'natural' ? {} : { order }),
      fields: shown,
      limit: 200,
    });
  };

  // O recorte do Panorama já abre com o resultado (uma vez, quando o catálogo chega).
  const sliceRan = useRef(false);
  useEffect(() => {
    if (!slice || sliceRan.current || !built.query || catalog.length === 0) return;
    sliceRan.current = true;
    setArgs({ filters: built.query, sort_by: sortBy, fields: shown, limit: 200 });
  }, [slice, built.query, catalog.length, sortBy, shown]);

  const load = (s: Screen) => {
    const d = fromScreenQuery(s.filters, catalog);
    const saved = s.filters.tags ?? [];
    setTags(saved);
    setDraft(d);
    setDraftKey((k) => k + 1);
    setCurrentId(s.id);
    setErrors({});
    setLastChange(null);
    const parsedQ = toScreenQuery(d, catalog).query;
    const q = parsedQ && saved.length ? { ...parsedQ, tags: saved } : parsedQ;
    if (q)
      setArgs({
        filters: q,
        sort_by: sortBy,
        fields: resultColumns(columns, filterFields(q), sortBy),
        limit: 200,
      });
  };

  const afterWrite = async (rowId: string, message: string) => {
    toast({ tone: 'success', title: message });
    const id = await lastChangeOf(rowId).catch(() => null);
    setLastChange(id ? { id, message } : null);
  };

  const saveNew = (name: string, description: string) => {
    if (!built.query) {
      setErrors(built.errors);
      setSaving(false);
      return;
    }
    create.mutate(
      { name, ...(description ? { description } : {}), filters: built.query },
      {
        onSuccess: (s) => {
          setSaving(false);
          setCurrentId(s.id);
          void afterWrite(s.id, t('pesquisa-scr-tela-salva', { nome: s.name }));
        },
      },
    );
  };

  const saveChanges = () => {
    setErrors(built.errors);
    if (!current || !built.query) return;
    update.mutate(
      { id: current.id, filters: built.query },
      {
        onSuccess: (s) =>
          void afterWrite(s.id, t('pesquisa-scr-tela-atualizada', { nome: s.name })),
      },
    );
  };

  const deleteCurrent = () => {
    if (!current) return Promise.resolve();
    const { id, name } = current;
    return remove.mutateAsync({ id }).then(
      () => {
        setCurrentId(null);
        return afterWrite(id, t('pesquisa-scr-tela-excluida', { nome: name }));
      },
      (e: Error) =>
        toast({ tone: 'error', title: t('pesquisa-scr-erro-excluir'), description: e.message }),
    );
  };

  const undoLast = () => {
    if (!lastChange) return;
    undo.mutate(
      { id: lastChange.id },
      {
        onSuccess: () => {
          toast({ tone: 'success', title: t('pesquisa-scr-desfeita') });
          setLastChange(null);
        },
        onError: (e) =>
          toast({ tone: 'error', title: t('pesquisa-scr-erro-desfazer'), description: e.message }),
      },
    );
  };

  const universeEmpty = status.data?.universe_size === 0 && result.data?.total === 0;
  const sortOptions = catalog
    .filter((i) => i.higher_is_better != null || i.key === 'price')
    .map((i) => ({ value: i.key, label: i.label, hint: GROUP_LABEL[i.group] }));

  const tableColumns: TechColumn<ScreenRow>[] = [
    {
      key: 'ticker',
      header: t('pesquisa-scr-ticker'),
      width: '96px',
      cell: (r) => <TickerLink assetId={r.asset_id} ticker={r.ticker} />,
      sortValue: (r) => r.ticker,
    },
    {
      key: 'name',
      header: t('pesquisa-scr-nome-col'),
      width: '220px',
      cell: (r) => (
        <span className="flex flex-col font-sans leading-tight">
          <span className="truncate">{r.name}</span>
          <span className="truncate text-[11.5px] text-dim">
            {[CLASS_LABEL[r.class], r.segment ?? r.sector].filter(Boolean).join(' · ')}
          </span>
        </span>
      ),
      sortValue: (r) => r.name,
    },
    ...metricColumns(
      (args?.fields ?? shown).filter((k) => catalog.some((i) => i.key === k)),
      catalog,
    ),
    {
      key: 'tags',
      header: t('pesquisa-tags'),
      width: '220px',
      cell: (r) => <TagLabels keys={r.tags} tags={allTags} />,
      sortValue: (r) => r.tags.length,
    },
    {
      key: 'methods',
      header: t('pesquisa-scr-metodos'),
      width: '170px',
      cell: (r) => <MethodBadges keys={badges.data?.[r.ticker] ?? []} names={methodNames} />,
      sortValue: (r) => badges.data?.[r.ticker]?.length ?? 0,
    },
  ];

  if (catalogQ.isPending) return <LoadingBlock />;
  if (catalogQ.isError) return <ErrorBlock error={catalogQ.error} />;

  return (
    <div className="flex flex-col gap-4">
      {status.data?.universe_size === 0 ? (
        <Notice tone="warning" title={t('pesquisa-scr-universo-vazio')}>
          {t('pesquisa-scr-universo-vazio-texto')}
        </Notice>
      ) : null}
      <InstrumentCard
        index="01"
        title={current ? t('pesquisa-scr-tela', { nome: current.name }) : t('pesquisa-scr-filtros')}
        headingLevel={3}
        actions={
          <div className="flex items-center gap-1.5">
            <div className="w-56">
              <Select
                aria-label={t('pesquisa-scr-telas-salvas')}
                value={current?.id ?? ''}
                placeholder={
                  screens.length ? t('pesquisa-scr-abrir-tela') : t('pesquisa-scr-nenhuma-tela')
                }
                disabled={!screens.length}
                onValueChange={(id) => {
                  const s = screens.find((x) => x.id === id);
                  if (s) load(s);
                }}
                options={screens.map((s) => ({ value: s.id, label: s.name }))}
              />
            </div>
          </div>
        }
        footer={
          current ? (
            <p className="text-[12px] text-dim">
              {current.description ? `${current.description} · ` : ''}
              {t('pesquisa-scr-salva-em', {
                resumo: current.summary,
                data: safeDate(current.updated_at.slice(0, 10)),
              })}
            </p>
          ) : undefined
        }
      >
        <div className="flex flex-col gap-5">
          <ConditionBuilder
            key={draftKey}
            draft={draft}
            onChange={setDraft}
            catalog={catalog}
            errors={errors}
          />
          <TagPicker
            tags={tagsFor(allTags, draft.universe.classes)}
            value={tags}
            counts={args ? result.data?.tag_counts : undefined}
            onChange={(next) => {
              setTags(next);
              // Com um resultado na tela, refaz na hora com as características novas.
              if (args && parsed.query)
                setArgs({
                  ...args,
                  filters: next.length ? { ...parsed.query, tags: next } : parsed.query,
                });
            }}
          />
          <UniverseFilters
            key={`u${draftKey}`}
            value={draft.universe}
            onChange={(universe) => setDraft((d) => ({ ...d, universe }))}
          />
          <div className="flex flex-wrap items-end gap-3">
            <Field label={t('pesquisa-ordenar-por')} className="w-64">
              <Select value={sortBy} onValueChange={setSortBy} options={sortOptions} />
            </Field>
            <Segmented<'natural' | SortOrder>
              label={t('pesquisa-scr-ordem')}
              value={order}
              onChange={setOrder}
              options={[
                { value: 'natural', label: t('pesquisa-ordem-natural') },
                { value: 'desc', label: t('pesquisa-maior-primeiro') },
                { value: 'asc', label: t('pesquisa-menor-primeiro') },
              ]}
            />
          </div>
          <ColumnsPicker catalog={catalog} value={columns} onChange={setColumns} />
          <div className="flex flex-wrap items-center gap-2 border-t border-line pt-3">
            <Button variant="primary" icon={<Icon name="search" size={12} />} onClick={run}>
              {t('pesquisa-scr-rodar')}
            </Button>
            {current ? (
              <Button loading={update.isPending} onClick={saveChanges}>
                {t('pesquisa-scr-salvar-alteracoes')}
              </Button>
            ) : null}
            {current ? (
              <Check
                label={t('pesquisa-scr-avisar')}
                checked={current.alert}
                disabled={update.isPending}
                onChange={(alert) => update.mutate({ id: current.id, alert })}
              />
            ) : null}
            <Button
              variant={current ? 'ghost' : 'secondary'}
              onClick={() => {
                setErrors(built.errors);
                if (built.query) {
                  create.reset();
                  setSaving(true);
                }
              }}
            >
              {current ? t('pesquisa-scr-salvar-nova') : t('pesquisa-scr-salvar-tela')}
            </Button>
            {current ? (
              <>
                <Button
                  variant="ghost"
                  onClick={() => {
                    setCurrentId(null);
                    setDraft(exampleDraft());
                    setDraftKey((k) => k + 1);
                    setLastChange(null);
                  }}
                >
                  {t('pesquisa-scr-nova')}
                </Button>
                <ConfirmButton
                  label={t('pesquisa-scr-excluir')}
                  title={t('pesquisa-scr-excluir')}
                  description={t('pesquisa-scr-excluir-texto', { nome: current.name })}
                  pending={remove.isPending}
                  onConfirm={deleteCurrent}
                />
              </>
            ) : null}
            {Object.keys(built.errors).length && Object.keys(errors).length ? (
              <span role="status" className="text-[12px] text-negative">
                {t('pesquisa-scr-corrija')}
              </span>
            ) : null}
          </div>
          {update.error ? <FormError error={update.error} /> : null}
          {lastChange ? (
            <Notice
              title={lastChange.message}
              actions={
                <Button size="sm" loading={undo.isPending} onClick={undoLast}>
                  {t('pesquisa-scr-desfazer')}
                </Button>
              }
            >
              {t('pesquisa-scr-desfazer-texto')}
            </Notice>
          ) : null}
        </div>
      </InstrumentCard>

      {args == null ? (
        <p className="text-[12.5px] text-dim">
          {withSlots(t('pesquisa-scr-instrucoes', { botao: slot(0) }), [
            <span key="botao" className="text-fg">
              {t('pesquisa-scr-rodar')}
            </span>,
          ])}
        </p>
      ) : result.isPending ? (
        <LoadingBlock label={t('pesquisa-scr-rodando')} />
      ) : result.isError ? (
        <ErrorBlock error={result.error} onRetry={() => void result.refetch()} />
      ) : universeEmpty ? (
        <NoUniverse what="screener" />
      ) : (
        <section aria-label={t('pesquisa-scr-resultado')} className="flex flex-col gap-2">
          <header className="flex flex-wrap items-baseline gap-3 px-1">
            <h3 className="label-caps text-fg">
              {t('pesquisa-scr-n-papeis', { n: result.data.total })}
            </h3>
            <span className="text-[12.5px] text-dim">{result.data.description}</span>
            <span className="ml-auto text-[12px] text-dim">
              {t('pesquisa-universo-de', { data: safeDate(result.data.as_of) })}
            </span>
          </header>
          <div
            className={cx(
              'rounded-card border border-line bg-surface transition-opacity',
              result.isFetching && 'opacity-70',
            )}
          >
            <TechTable
              label={t('pesquisa-scr-resultado')}
              prefsId="Resultado do screener"
              rows={result.data.items}
              columns={tableColumns}
              getRowId={(r) => r.ticker}
              onActivate={(r) => void navigate(subjectHref(r.asset_id, r.ticker))}
              empty={t('pesquisa-scr-nenhum')}
              maxHeight="640px"
              dense
            />
          </div>
          {result.data.total > result.data.items.length ? (
            <p className="text-[12px] text-dim">
              {t('pesquisa-scr-mostrando', { n: result.data.items.length })}
            </p>
          ) : null}
        </section>
      )}
      <SaveDialog
        key={saving ? 'open' : 'closed'}
        open={saving}
        onOpenChange={setSaving}
        onSave={saveNew}
        pending={create.isPending}
        error={create.error}
      />
    </div>
  );
}
