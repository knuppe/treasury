/**
 * Carteira › Saúde dos ativos › Dados dos ativos (docs/agent-enrichment.md §10.1): os campos
 * do cadastro que as fontes do cofre sabem completar (`assets.data_gaps`), com fonte, data,
 * confiança, conflitos e divergências. Nada é gravado sem marcar: de saída só vêm marcados
 * os campos vazios de confiança alta (Q9); trocar um valor seu é uma marca à parte. Gravar
 * passa pelo histórico de alterações (desfazer).
 */
import { useEffect, useMemo, useRef, useState } from 'react';
import { Link, useSearchParams } from 'react-router';

import type { AssetData } from '@/api/generated/AssetData';
import type { EnrichItem } from '@/api/generated/EnrichItem';
import type { FieldCandidate } from '@/api/generated/FieldCandidate';
import type { FieldStatus } from '@/api/generated/FieldStatus';
import { useDataGaps, useEnrichApply, useEnrichLookupAll } from '@/api/hooks/assetData';
import { Button, toast } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

import { ErrorBlock, LoadingBlock } from '../../portfolio/components/States';
import { Segmented } from '../../portfolio/components/Segmented';
import { ConfidenceChip, fieldLabel, itemFor, SourceChip, stateLabel, valueLabel } from './parts';

type Filter = 'all' | 'fill' | 'conflict' | 'diverge';

interface Row {
  key: string;
  asset: AssetData;
  field: FieldStatus;
}

/** Valor marcado por linha (campo de um ativo): a sugestão ou a opção escolhida. */
type Picks = Record<string, string>;

const rowKey = (assetId: string, field: string) => `${assetId}:${field}`;

function defaults(rows: Row[]): Picks {
  const out: Picks = {};
  for (const r of rows) {
    const c = r.field.candidate;
    if (r.field.state === 'fill' && c?.band === 'high') out[r.key] = c.value;
  }
  return out;
}

export function DataGapsPanel() {
  const q = useDataGaps();
  const apply = useEnrichApply();
  const lookupAll = useEnrichLookupAll();
  const [filter, setFilter] = useState<Filter>('all');
  const [picks, setPicks] = useState<Picks | null>(null);
  // `?secao=dados` (alerta, aviso da Carteira, ⌘K): rola até aqui quando carregar.
  const [params] = useSearchParams();
  const ref = useRef<HTMLElement>(null);
  const loaded = Boolean(q.data);
  useEffect(() => {
    if (loaded && params.get('secao') === 'dados') {
      ref.current?.scrollIntoView?.({ block: 'start' });
    }
  }, [loaded, params]);

  const rows = useMemo<Row[]>(
    () =>
      (q.data?.assets ?? []).flatMap((asset) =>
        asset.fields.map((field) => ({ key: rowKey(asset.asset_id, field.field), asset, field })),
      ),
    [q.data],
  );
  const chosen = picks ?? defaults(rows);

  if (q.isPending) return <LoadingBlock label={t('dados-titulo')} />;
  if (q.isError) return <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />;
  const s = q.data.summary;
  const visible = rows.filter(
    (r) =>
      filter === 'all' ||
      r.field.state === filter ||
      (filter === 'fill' && r.field.state === 'update'),
  );

  const set = (key: string, value: string | null) => {
    const next = { ...chosen };
    if (value == null) delete next[key];
    else next[key] = value;
    setPicks(next);
  };

  const items: EnrichItem[] = rows.flatMap((r) => {
    const v = chosen[r.key];
    if (v == null) return [];
    const c =
      r.field.candidate?.value === v
        ? r.field.candidate
        : r.field.options.find((o) => o.value === v);
    return c ? [itemFor(r.asset.asset_id, r.field, c)] : [];
  });

  const save = () =>
    apply.mutate(
      { items },
      {
        onSuccess: (res) => {
          setPicks(null);
          toast({
            tone: 'success',
            title: t('dados-aplicado', { n: res.fields }),
            description: t('dados-aplicado-desfazer'),
          });
        },
        onError: (e: Error) =>
          toast({ tone: 'error', title: t('dados-nao-aplicou'), description: e.message }),
      },
    );

  return (
    <section
      ref={ref}
      id="dados-dos-ativos"
      className="mt-6 flex flex-col gap-3 border-t border-line pt-5"
      aria-label={t('dados-titulo')}
    >
      <div className="flex flex-col gap-1">
        <h3 className="label-caps text-fg">{t('dados-titulo')}</h3>
        <p className="text-[12.5px] leading-snug text-dim">{t('dados-ajuda')}</p>
        <div className="flex flex-wrap items-center gap-2">
          <p className="num grow text-[12.5px] text-fg">
            {t('dados-resumo', {
              ativos: s.assets,
              sem_setor: s.missing_sector,
              sem_isin: s.missing_isin,
              conflitos: s.conflicts,
            })}
          </p>
          <Button
            size="sm"
            variant="ghost"
            loading={lookupAll.isPending}
            title={t('dados-consultar-dica')}
            onClick={() =>
              lookupAll.mutate(
                {},
                {
                  onSuccess: (r) =>
                    toast({
                      tone: r.errors > 0 ? 'warning' : 'success',
                      title: t('dados-consultados-todos', { ativos: r.assets, n: r.observations }),
                      description:
                        r.errors > 0 ? t('dados-consultados-erros', { n: r.errors }) : undefined,
                    }),
                  onError: (e: Error) =>
                    toast({
                      tone: 'error',
                      title: t('dados-nao-consultou'),
                      description: e.message,
                    }),
                },
              )
            }
          >
            {t('dados-consultar-todos')}
          </Button>
        </div>
      </div>
      {rows.length === 0 ? (
        <p className="text-[12.5px] text-dim">{t('dados-nada')}</p>
      ) : (
        <>
          <div className="flex flex-wrap items-center gap-2">
            <Segmented<Filter>
              size="sm"
              label={t('dados-titulo')}
              value={filter}
              onChange={setFilter}
              options={[
                { value: 'all', label: t('dados-filtro-todos') },
                { value: 'fill', label: t('dados-filtro-vazios') },
                { value: 'conflict', label: t('dados-filtro-conflitos') },
                { value: 'diverge', label: t('dados-filtro-divergencias') },
              ]}
            />
            <span className="grow" />
            <Button size="sm" variant="ghost" onClick={() => setPicks(defaults(rows))}>
              {t('dados-marcar-altos')}
            </Button>
            <Button size="sm" variant="ghost" onClick={() => setPicks({})}>
              {t('dados-desmarcar')}
            </Button>
          </div>
          <div className="overflow-x-auto">
            <table className="w-full min-w-[720px] text-[13px]">
              <thead>
                <tr className="label-caps border-b border-line text-left">
                  <th className="py-1.5 pr-2 font-normal">{t('dados-col-ativo')}</th>
                  <th className="py-1.5 pr-2 font-normal">{t('dados-col-campo')}</th>
                  <th className="py-1.5 pr-2 font-normal">{t('dados-col-atual')}</th>
                  <th className="py-1.5 pr-2 font-normal">{t('dados-col-sugestao')}</th>
                  <th className="py-1.5 pr-2 font-normal">{t('dados-col-fonte')}</th>
                  <th className="py-1.5 font-normal">{t('dados-col-confianca')}</th>
                </tr>
              </thead>
              <tbody>
                {visible.map((r, i) => (
                  <GapRow
                    key={r.key}
                    row={r}
                    first={i === 0 || visible[i - 1]?.asset.asset_id !== r.asset.asset_id}
                    picked={chosen[r.key] ?? null}
                    onPick={(v) => set(r.key, v)}
                  />
                ))}
              </tbody>
            </table>
          </div>
          <div className="flex flex-wrap items-center gap-3">
            <p className="max-w-xl grow text-[12px] leading-snug text-dim">{t('dados-rodape')}</p>
            <Button
              size="sm"
              disabled={items.length === 0}
              loading={apply.isPending}
              onClick={save}
            >
              {t('dados-aplicar-marcados', { n: items.length })}
            </Button>
          </div>
        </>
      )}
    </section>
  );
}

function GapRow({
  row,
  first,
  picked,
  onPick,
}: {
  row: Row;
  first: boolean;
  picked: string | null;
  onPick: (v: string | null) => void;
}) {
  const { asset, field } = row;
  const label = asset.ticker ?? asset.name;
  const id = `dado-${row.key}`;
  const candidates: FieldCandidate[] =
    field.state === 'conflict' ? field.options : field.candidate ? [field.candidate] : [];
  return (
    <tr className={cx('align-top', first && 'border-t border-line/60')}>
      <td className="py-1.5 pr-2">
        {first ? (
          <Link
            to={`/ativos?ativo=${encodeURIComponent(asset.asset_id)}&secao=cadastro`}
            className="font-medium text-fg underline-offset-2 hover:text-accent hover:underline"
          >
            {label}
          </Link>
        ) : null}
      </td>
      <td className="py-1.5 pr-2 text-dim">
        {fieldLabel(field.field)}
        <span className="block text-[11px]">{stateLabel(field.state)}</span>
      </td>
      <td className="max-w-40 py-1.5 pr-2 break-words text-dim">
        {field.current != null ? (
          valueLabel(field.field, field.current)
        ) : (
          <span className="italic">{t('dados-vazio')}</span>
        )}
      </td>
      <td className="py-1.5 pr-2" colSpan={field.state === 'conflict' ? 3 : 1}>
        {field.state === 'conflict' ? (
          <fieldset className="flex flex-col gap-1">
            <legend className="text-[11px] text-warning">{t('dados-escolha')}</legend>
            {candidates.map((c) => (
              <label key={c.value} className="flex flex-wrap items-baseline gap-2">
                <input
                  type="radio"
                  name={id}
                  checked={picked === c.value}
                  onChange={() => onPick(c.value)}
                />
                <span className="text-fg">{valueLabel(field.field, c.value)}</span>
                <SourceChip c={c} />
                <ConfidenceChip c={c} />
              </label>
            ))}
            {picked ? (
              <button
                type="button"
                className="w-fit text-[11px] text-dim underline"
                onClick={() => onPick(null)}
              >
                {t('dados-desmarcar')}
              </button>
            ) : null}
          </fieldset>
        ) : field.candidate ? (
          <label className="flex items-baseline gap-2">
            <input
              type="checkbox"
              checked={picked === field.candidate.value}
              onChange={(e) => onPick(e.target.checked ? field.candidate!.value : null)}
              aria-label={`${label} · ${fieldLabel(field.field)}`}
            />
            <span className="flex flex-col">
              <span className="break-words text-fg">
                {valueLabel(field.field, field.candidate.value)}
              </span>
              {field.state === 'diverge' ? (
                <span className="text-[11px] text-warning">{t('dados-trocar-seu')}</span>
              ) : null}
              {field.effect ? <span className="text-[11px] text-dim">{field.effect}</span> : null}
            </span>
          </label>
        ) : null}
      </td>
      {field.state !== 'conflict' && field.candidate ? (
        <>
          <td className="py-1.5 pr-2">
            <SourceChip c={field.candidate} />
          </td>
          <td className="py-1.5">
            <ConfidenceChip c={field.candidate} />
          </td>
        </>
      ) : field.state !== 'conflict' ? (
        <>
          <td />
          <td />
        </>
      ) : null}
    </tr>
  );
}
