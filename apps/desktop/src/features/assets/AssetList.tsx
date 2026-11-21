/**
 * Lista e busca de ativos cadastrados (ticker ou nome), com filtro por classe, só
 * os que têm operações e arquivados. Enter ou duplo clique abre a página do ativo. A coluna
 * Score traz a nota de saúde de cada ativo (`score.assets`, o cálculo do dia) e abre a aba
 * Saúde do ativo.
 */
import { useDeferredValue, useEffect, useLayoutEffect, useMemo, useRef } from 'react';
import { create } from 'zustand';
import { useNavigate } from 'react-router';

import { useAssets } from '@/api/hooks/assets';
import { useScoreAssets } from '@/api/hooks/score';
import type { Asset } from '@/api/generated/Asset';
import type { AssetScoreSummary } from '@/api/generated/AssetScoreSummary';
import type { AssetClass } from '@/api/generated/AssetClass';
import type { PositionFilter } from '@/api/generated/PositionFilter';
import { EmptyState, Icon, Input, Select, TechTable, type TechColumn } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

import { Check } from '../portfolio/components/Fields';
import { ErrorBlock, LoadingBlock } from '../portfolio/components/States';
import { CLASS_LABEL, CLASS_ORDER, quoteSourceLabel, termsSummary } from '../portfolio/labels';
import { BAND, bandKey, formatScore } from '../score/format';
import { scoreHref } from '../score/ScorePortfolioPanel';
import { assetMenuItems } from './assetMenu';

const ALL = '__all__';

type Scores = { byId: Map<string, AssetScoreSummary>; loading: boolean };

/** Texto de apoio da nota (faixa, confiança, o que falta). */
function scoreTitle(r: AssetScoreSummary): string {
  if (r.status === 'not_supported') {
    // Título vencido e outros casos sem o que avaliar trazem o motivo na faixa.
    // A faixa e o motivo vêm do núcleo; "Dados insuficientes" é o valor do núcleo para "sem faixa".
    return (
      r.error ??
      (r.band_label !== 'Dados insuficientes' ? r.band_label : t('ativos-score-classe-sem-score'))
    );
  }
  if (r.status === 'insufficient_data') {
    return r.missing_critical.length
      ? t('ativos-score-falta', { falta: r.missing_critical.join(', ') })
      : t('score-dados-insuficientes');
  }
  const conf = Math.round(r.confidence * 100);
  return r.status === 'partial'
    ? t('ativos-score-parcial', { pct: conf })
    : t('ativos-score-faixa', { faixa: r.band_label, pct: conf });
}

function ScoreCell({
  r,
  loading,
  onOpen,
}: {
  r: AssetScoreSummary | undefined;
  loading: boolean;
  onOpen: (r: AssetScoreSummary) => void;
}) {
  if (!r) return <span className="text-dim">{loading ? '…' : '—'}</span>;
  const scored = r.score != null && (r.status === 'ok' || r.status === 'partial');
  const band = BAND[bandKey(r.status === 'ok' ? r.band : 'dados_insuficientes')];
  return (
    <button
      type="button"
      title={scoreTitle(r)}
      aria-label={
        scored
          ? t('ativos-score-nota', { nota: formatScore(r.score), detalhe: scoreTitle(r) })
          : t('ativos-score-sem-nota', { detalhe: scoreTitle(r) })
      }
      className={cx(
        'inline-flex items-center gap-1 rounded-control px-1 hover:bg-surface-2',
        scored ? band.text : 'text-dim',
      )}
      onClick={(e) => {
        e.stopPropagation();
        onOpen(r);
      }}
    >
      {scored ? (
        <>
          <span aria-hidden className="text-[10px]">
            {r.status === 'partial' ? '≈' : band.mark}
          </span>
          <span className="num font-medium">{formatScore(r.score)}</span>
        </>
      ) : r.status === 'insufficient_data' ? (
        <span className="text-[11.5px]">{t('ativos-score-sem-dados')}</span>
      ) : (
        <span>—</span>
      )}
    </button>
  );
}

const scoreColumn = (
  scores: Scores,
  onOpen: (r: AssetScoreSummary) => void,
): TechColumn<Asset> => ({
  key: 'score',
  get header() {
    return t('ativos-col-score');
  },
  width: '8%',
  cell: (a) => <ScoreCell r={scores.byId.get(a.id)} loading={scores.loading} onOpen={onOpen} />,
  // Sem nota vai para o fim.
  sortValue: (a) => scores.byId.get(a.id)?.score ?? null,
});

const COLUMNS: TechColumn<Asset>[] = [
  {
    key: 'ticker',
    get header() {
      return t('ativos-col-ticker');
    },
    width: '110px',
    cell: (a) => <span className="font-medium text-fg">{a.ticker ?? '—'}</span>,
    // Sem código (CDB, Tesouro…), vai para o fim; o nome está na coluna ao lado.
    sortValue: (a) => a.ticker,
  },
  {
    key: 'name',
    get header() {
      return t('ativos-col-nome');
    },
    cell: (a) => (
      <span className="flex items-center gap-2 font-sans">
        <span
          className="truncate"
          title={
            a.official_name ? t('ativos-nome-oficial-de', { nome: a.official_name }) : undefined
          }
        >
          {a.name}
        </span>
        {a.archived ? (
          <span className="label-caps rounded-control border border-line px-1 text-[10px]">
            {t('ativos-arquivado')}
          </span>
        ) : null}
      </span>
    ),
    sortValue: (a) => a.name,
  },
  {
    key: 'class',
    get header() {
      return t('ativos-col-classe');
    },
    width: '13%',
    cell: (a) => <span className="block truncate font-sans text-dim">{a.class_label}</span>,
    sortValue: (a) => CLASS_ORDER.indexOf(a.class),
  },
  {
    key: 'market',
    get header() {
      return t('ativos-col-mercado');
    },
    width: '10%',
    cell: (a) => <span className="block truncate">{a.market ?? '—'}</span>,
  },
  {
    key: 'currency',
    get header() {
      return t('ativos-col-moeda');
    },
    width: '7%',
    cell: (a) => a.currency,
  },
  {
    key: 'detail',
    get header() {
      return t('ativos-col-setor-termos');
    },
    width: '20%',
    cell: (a) => {
      const text = a.fixed_income
        ? termsSummary(a.fixed_income)
        : [a.sector, a.segment].filter(Boolean).join(' · ') || '—';
      return (
        <span
          title={text}
          className="block w-0 min-w-full truncate font-sans text-[12.5px] text-dim"
        >
          {text}
        </span>
      );
    },
  },
  {
    key: 'source',
    get header() {
      return t('ativos-col-cotacao');
    },
    width: '12%',
    cell: (a) => (
      <span className="block w-0 min-w-full truncate font-sans text-[12px] text-dim">
        {quoteSourceLabel(a)}
      </span>
    ),
  },
];

type PositionChoice = PositionFilter | typeof ALL;

/**
 * Filtros e rolagem da lista, em memória: abrir um ativo e voltar (trilha, Voltar ou o menu)
 * devolve a lista como estava. A ordenação já fica nas preferências da tabela.
 */
interface AssetListUi {
  text: string;
  cls: string;
  position: PositionChoice;
  archived: boolean;
  scrollTop: number;
  set: (patch: Partial<Omit<AssetListUi, 'set'>>) => void;
}

export const ASSET_LIST_DEFAULTS = {
  text: '',
  cls: ALL as string,
  // Padrão: só o que está em carteira; encerradas e o resto do cadastro pelo filtro.
  position: 'open' as PositionChoice,
  archived: false,
  scrollTop: 0,
};

export const useAssetListUi = create<AssetListUi>()((set) => ({
  ...ASSET_LIST_DEFAULTS,
  set: (patch) => set(patch),
}));

export function AssetList({ onOpen }: { onOpen: (asset: Asset) => void }) {
  const { text, cls, position, archived, set } = useAssetListUi();
  const deferred = useDeferredValue(text.trim());
  const navigate = useNavigate();
  const q = useAssets({
    ...(deferred ? { text: deferred } : {}),
    ...(cls !== ALL ? { classes: [cls as AssetClass] } : {}),
    ...(position !== ALL ? { position } : {}),
    ...(archived ? { include_archived: true } : {}),
  });
  const ids = useMemo(() => (q.data ?? []).map((a) => a.id), [q.data]);
  const scoreQ = useScoreAssets(ids);
  const columns = useMemo(() => {
    const byId = new Map((scoreQ.data ?? []).flatMap((r) => (r.asset_id ? [[r.asset_id, r]] : [])));
    const col = scoreColumn(
      { byId, loading: scoreQ.isFetching },
      (r) => void navigate(scoreHref(r)),
    );
    // Depois do nome: a nota é o resumo do ativo.
    return [...COLUMNS.slice(0, 2), col, ...COLUMNS.slice(2)];
  }, [scoreQ.data, scoreQ.isFetching, navigate]);
  // Rolagem da tabela: guardada ao sair, devolvida quando as linhas voltam.
  const box = useRef<HTMLDivElement>(null);
  const restored = useRef(false);
  const loaded = q.data !== undefined;
  useLayoutEffect(() => {
    if (restored.current || !loaded) return;
    restored.current = true;
    const el = box.current?.querySelector<HTMLElement>('.tech-table');
    const top = useAssetListUi.getState().scrollTop;
    if (el && top) el.scrollTop = top;
  }, [loaded]);
  const scrolled = useRef<number | null>(null);
  useEffect(
    () => () => {
      if (scrolled.current != null) useAssetListUi.getState().set({ scrollTop: scrolled.current });
    },
    [],
  );
  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center gap-3">
        <div className="w-72">
          <Input
            type="search"
            aria-label={t('ativos-buscar-rotulo')}
            placeholder={t('ativos-buscar')}
            value={text}
            onChange={(e) => set({ text: e.target.value, scrollTop: 0 })}
            prefix={<Icon name="search" size={13} />}
          />
        </div>
        <div className="w-52">
          <Select
            aria-label={t('ativos-filtrar-classe')}
            value={cls}
            onValueChange={(v) => set({ cls: v, scrollTop: 0 })}
            options={[
              { value: ALL, label: t('ativos-todas-classes') },
              ...CLASS_ORDER.map((c) => ({ value: c, label: CLASS_LABEL[c] })),
            ]}
          />
        </div>
        <div className="w-56">
          <Select
            aria-label={t('ativos-filtrar-posicao')}
            value={position}
            onValueChange={(v) => set({ position: v as PositionChoice, scrollTop: 0 })}
            options={[
              { value: 'open', label: t('ativos-posicao-aberta') },
              { value: 'closed', label: t('ativos-posicoes-encerradas') },
              { value: 'traded', label: t('ativos-posicao-com-operacoes') },
              { value: ALL, label: t('ativos-posicao-todos') },
            ]}
          />
        </div>
        <Check
          checked={archived}
          onChange={(v) => set({ archived: v, scrollTop: 0 })}
          label={t('ativos-incluir-arquivados')}
        />
        <span className="ml-auto text-[12px] text-dim">
          {q.data ? t('ativos-n-ativos', { n: q.data.length }) : ''}
        </span>
      </div>
      {q.isPending ? (
        <LoadingBlock />
      ) : q.isError ? (
        <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />
      ) : q.data.length === 0 && !deferred && cls === ALL && position === ALL ? (
        <EmptyState title={t('ativos-vazio-titulo')} description={t('ativos-vazio-descricao')} />
      ) : (
        <div
          ref={box}
          className="rounded-card border border-line bg-surface"
          onScrollCapture={(e) => {
            const el = e.target as HTMLElement;
            if (el.classList.contains('tech-table')) scrolled.current = el.scrollTop;
          }}
        >
          <TechTable
            className="[&_table]:table-fixed"
            label={t('ativos-tabela')}
            prefsId="Ativos cadastrados"
            rows={q.data}
            columns={columns}
            getRowId={(a) => a.id}
            onActivate={onOpen}
            empty={t('ativos-nenhum-filtro')}
            maxHeight="620px"
            rowMenuLabel={(a) => t('ativos-acoes-de', { nome: a.ticker ?? a.name })}
            rowMenu={(a) => assetMenuItems(a, (path) => void navigate(path))}
          />
        </div>
      )}
    </div>
  );
}
