/**
 * 08 · Carteira › Bens: patrimônio físico. Resumo (valor, custo, ganho, saldo devedor e
 * patrimônio dos bens), a lista com valor, ganho e patrimônio de cada bem e o detalhe do
 * escolhido.
 *
 * Estado na URL (junto de `?aba=bens`): `bem=<id>` escolhe o bem; `novo=bem` abre o
 * cadastro; `bens=arquivados` lista também os arquivados e vendidos; `acao=benfeitoria |
 * avaliacao | venda | financiamento | editar` abre o formulário do bem escolhido (usado pela
 * barra de comando).
 */
import { useCallback, useMemo, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router';

import { useMembers } from '@/api/hooks/household';
import {
  useDeletePhysical,
  usePhysicalAssets,
  usePhysicalSummary,
  useUpdatePhysical,
} from '@/api/hooks/physical';
import { useUndoToast } from '@/api/hooks/undoToast';
import type { PhysicalAssetView } from '@/api/generated/PhysicalAssetView';
import type { PhysicalSummary } from '@/api/generated/PhysicalSummary';
import { useRegisterCommands, type AppCommand } from '@/commands/registry';
import {
  Button,
  EmptyState,
  Icon,
  InstrumentCard,
  Measure,
  Money,
  TechTable,
  toast,
  type ContextMenuEntry,
} from '@/design/components';
import { Pct } from '@/features/portfolio/components/Pct';
import { Checkbox, ConfirmDialog, QueryState } from '@/features/transactions/ui/controls';
import { errorText } from '@/features/transactions/ui/errors';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';

import { KIND_LABEL } from './labels';
import { AssetFormDialog } from './PhysicalDialogs';
import { PhysicalDetail, type DetailDialog } from './PhysicalDetail';
import { methodText, ownersText, totalsByKind } from './transform';

const ACTIONS: Record<string, DetailDialog> = {
  editar: 'edit',
  benfeitoria: 'improvement',
  avaliacao: 'valuation',
  venda: 'sale',
  financiamento: 'loan',
};
const ACTION_OF: Record<Exclude<DetailDialog, null>, string> = {
  edit: 'editar',
  improvement: 'benfeitoria',
  valuation: 'avaliacao',
  sale: 'venda',
  loan: 'financiamento',
};

export function PhysicalSummaryCards({ summary }: { summary: PhysicalSummary }) {
  const kinds = totalsByKind(summary.assets);
  const live = summary.assets.filter((a) => !a.figures.sold).length;
  return (
    <div className="grid gap-4 @min-[530px]:grid-cols-3">
      <InstrumentCard index="07.A" title={t('bens-resumo-valor')} headingLevel={3}>
        <Measure
          label={t('bens-n-bens', { n: live })}
          size="md"
          value={<Money cents={summary.total_value} estimated={summary.estimated} />}
          hint={kinds.map((k) => `${k.label} ${k.count}`).join(' · ') || undefined}
        />
      </InstrumentCard>
      <InstrumentCard index="07.B" title={t('bens-resumo-ganho')} headingLevel={3}>
        <Measure
          size="md"
          label={withSlots(t('bens-custo-de', { valor: slot(0) }), [
            <Money key="custo" cents={summary.total_cost_basis} />,
          ])}
          value={<Money cents={summary.total_gain} tone="auto" signed />}
          hint={
            <Pct
              value={
                summary.total_cost_basis ? summary.total_gain / summary.total_cost_basis : null
              }
              signed
              decimals={1}
            />
          }
        />
      </InstrumentCard>
      <InstrumentCard index="07.C" title={t('bens-resumo-patrimonio')} headingLevel={3}>
        <Measure
          label={t('bens-valor-menos-saldo-financiamentos')}
          size="md"
          value={<Money cents={summary.total_equity} estimated={summary.estimated} />}
          hint={
            summary.total_loan_balance > 0
              ? withSlots(t('bens-saldo-devedor-valor', { valor: slot(0) }), [
                  <Money key="saldo" cents={summary.total_loan_balance} />,
                ])
              : t('bens-sem-financiamentos')
          }
        />
      </InstrumentCard>
    </div>
  );
}

export function PhysicalTable({
  assets,
  selectedId,
  onSelect,
  rowMenu,
}: {
  assets: readonly PhysicalAssetView[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  rowMenu?: (a: PhysicalAssetView) => readonly ContextMenuEntry[];
}) {
  const members = useMembers().data ?? [];
  return (
    <TechTable<PhysicalAssetView>
      label={t('bens-titulo')}
      prefsId="Bens"
      rows={assets}
      getRowId={(a) => a.id}
      selectedId={selectedId}
      onSelectedChange={(id) => id && onSelect(id)}
      onActivate={(a) => onSelect(a.id)}
      rowMenu={rowMenu}
      rowMenuLabel={(a) => t('bens-acoes-de', { nome: a.name })}
      columns={[
        {
          key: 'name',
          header: t('bens-bem'),
          sortValue: (a) => a.name,
          cell: (a) => (
            <span className="flex flex-col">
              <span className="font-medium">{a.name}</span>
              <span className="text-[11.5px] text-dim">
                {KIND_LABEL[a.kind]} · {ownersText(a.member_ids, members)}
                {a.figures.sold
                  ? ` · ${t('bens-vendido')}`
                  : a.archived
                    ? ` · ${t('bens-arquivado')}`
                    : ''}
              </span>
            </span>
          ),
        },
        {
          key: 'method',
          header: t('bens-avaliacao'),
          cell: (a) => <span className="text-[12.5px] text-dim">{methodText(a)}</span>,
        },
        {
          key: 'cost',
          header: t('bens-custo'),
          numeric: true,
          sortValue: (a) => a.figures.cost_basis,
          cell: (a) => <Money cents={a.figures.cost_basis} />,
        },
        {
          key: 'value',
          header: t('bens-valor'),
          numeric: true,
          sortValue: (a) => a.figures.value,
          cell: (a) => <Money cents={a.figures.value} estimated={a.figures.estimated} />,
        },
        {
          key: 'gain',
          header: t('bens-ganho'),
          numeric: true,
          sortValue: (a) => a.figures.nominal_gain,
          cell: (a) => (
            <span className="flex flex-col items-end">
              <Money cents={a.figures.nominal_gain} tone="auto" signed />
              <span className="text-[11.5px] text-dim">
                <Pct value={a.figures.annualized} signed decimals={1} /> {t('bens-ao-ano-abrev')}
              </span>
            </span>
          ),
        },
        {
          key: 'equity',
          header: t('bens-patrimonio'),
          numeric: true,
          sortValue: (a) => a.figures.equity,
          cell: (a) => (
            <span className="flex flex-col items-end">
              <Money cents={a.figures.equity} />
              {a.loans.length > 0 ? (
                <span className="text-[11.5px] text-dim">
                  {'LTV'} <Pct value={a.figures.ltv} decimals={0} />
                </span>
              ) : null}
            </span>
          ),
        },
      ]}
    />
  );
}

export function PhysicalPanel() {
  const [params, setParams] = useSearchParams();
  const archived = params.get('bens') === 'arquivados';
  const summary = usePhysicalSummary();
  const assets = usePhysicalAssets({ include_archived: archived });
  const update = useCallback(
    (patch: Record<string, string | null>) =>
      setParams(
        (prev) => {
          const next = new URLSearchParams(prev);
          for (const [k, v] of Object.entries(patch)) {
            if (v == null) next.delete(k);
            else next.set(k, v);
          }
          return next;
        },
        { replace: true },
      ),
    [setParams],
  );
  const list = assets.data ?? [];
  const selected = list.find((a) => a.id === params.get('bem')) ?? list[0] ?? null;
  const creating = params.get('novo') === 'bem';
  const dialog = ACTIONS[params.get('acao') ?? ''] ?? null;

  const selId = selected?.id;
  const selName = selected?.name;
  const commands = useMemo<AppCommand[]>(
    () =>
      selId && selName
        ? [
            {
              id: 'physical.improvement',
              title: t('bens-cmd-benfeitoria', { nome: selName }),
              group: t('bens-cmd-grupo'),
              keywords: t('bens-cmd-benfeitoria-busca').split(','),
              perform: () => update({ acao: 'benfeitoria', bem: selId }),
            },
            {
              id: 'physical.valuation',
              title: t('bens-cmd-avaliacao', { nome: selName }),
              group: t('bens-cmd-grupo'),
              keywords: t('bens-cmd-avaliacao-busca').split(','),
              perform: () => update({ acao: 'avaliacao', bem: selId }),
            },
          ]
        : [],
    [selId, selName, update],
  );
  useRegisterCommands(commands);

  // Menu de contexto da linha: os mesmos caminhos dos botões do detalhe, pela URL.
  const navigate = useNavigate();
  const save = useUpdatePhysical();
  const remove = useDeletePhysical();
  const undoToast = useUndoToast();
  const [deleting, setDeleting] = useState<PhysicalAssetView | null>(null);
  const act = (a: PhysicalAssetView, acao: string) => update({ bem: a.id, acao });
  const rowMenu = (a: PhysicalAssetView): ContextMenuEntry[] => [
    { heading: a.name },
    {
      label: t('bens-abrir'),
      icon: 'arrowRight',
      onSelect: () => update({ bem: a.id, acao: null }),
    },
    { separator: true },
    {
      label: t('bens-registrar-benfeitoria-menu'),
      icon: 'plus',
      onSelect: () => act(a, 'benfeitoria'),
    },
    {
      label: t('bens-registrar-avaliacao-menu'),
      icon: 'plus',
      onSelect: () => act(a, 'avaliacao'),
    },
    {
      label: t('bens-novo-projeto-menu'),
      icon: 'plus',
      onSelect: () =>
        void navigate(`/orcamento?aba=projetos&novo=projeto&bem=${encodeURIComponent(a.id)}`),
    },
    {
      label: t('bens-ligar-financiamento-menu'),
      icon: 'sliders',
      onSelect: () => act(a, 'financiamento'),
    },
    !a.figures.sold && {
      label: t('bens-registrar-venda-menu'),
      icon: 'check',
      onSelect: () => act(a, 'venda'),
    },
    { separator: true },
    { label: t('bens-editar-cadastro-menu'), icon: 'sliders', onSelect: () => act(a, 'editar') },
    {
      label: a.archived ? t('bens-reativar') : t('bens-arquivar'),
      icon: a.archived ? 'restore' : 'minus',
      onSelect: () =>
        save.mutate(
          { id: a.id, archived: !a.archived },
          {
            onSuccess: () =>
              undoToast({ title: a.archived ? t('bens-reativado') : t('bens-arquivado-toast') }),
            onError: (e) =>
              toast({
                tone: 'error',
                title: t('bens-nao-foi-possivel'),
                description: errorText(e),
              }),
          },
        ),
    },
    { label: t('bens-excluir-menu'), icon: 'x', danger: true, onSelect: () => setDeleting(a) },
  ];

  return (
    <div className="flex flex-col gap-6">
      <QueryState
        loading={assets.isPending}
        error={assets.error ?? summary.error}
        onRetry={() => void assets.refetch()}
        label={t('bens-avaliando')}
      >
        {list.length === 0 && !archived ? (
          <EmptyState
            kicker={t('bens-vazio-kicker')}
            title={t('bens-vazio-titulo')}
            description={t('bens-vazio-descricao')}
            actions={
              <Button variant="primary" onClick={() => update({ novo: 'bem' })}>
                {t('bens-novo')}
              </Button>
            }
          />
        ) : (
          <>
            {summary.data ? <PhysicalSummaryCards summary={summary.data} /> : null}
            <div className="flex flex-col gap-2">
              <div className="flex flex-wrap items-center justify-between gap-3">
                <h3 className="label-caps text-fg">{t('bens-titulo')}</h3>
                <div className="flex flex-wrap items-center gap-3">
                  <Checkbox
                    checked={archived}
                    onChange={(v) => update({ bens: v ? 'arquivados' : null })}
                    label={t('bens-mostrar-arquivados')}
                  />
                  <Button
                    variant="primary"
                    icon={<Icon name="plus" size={12} />}
                    onClick={() => update({ novo: 'bem' })}
                  >
                    {t('bens-novo')}
                  </Button>
                </div>
              </div>
              <PhysicalTable
                assets={list}
                selectedId={selected?.id ?? null}
                onSelect={(id) => update({ bem: id })}
                rowMenu={rowMenu}
              />
            </div>
            {selected ? (
              <PhysicalDetail
                key={selected.id}
                asset={selected}
                dialog={dialog}
                onDialog={(d) => update({ acao: d ? ACTION_OF[d] : null, bem: selected.id })}
                onDeleted={() => update({ bem: null })}
              />
            ) : null}
          </>
        )}
      </QueryState>
      <ConfirmDialog
        open={deleting != null}
        onOpenChange={(o) => (!o ? setDeleting(null) : undefined)}
        title={t('bens-excluir-titulo', { nome: deleting?.name ?? t('bens-o-bem') })}
        description={t('bens-excluir-descricao-lista')}
        confirmLabel={t('bens-excluir-bem')}
        pending={remove.isPending}
        error={remove.error}
        onConfirm={() =>
          deleting &&
          remove.mutate(
            { id: deleting.id },
            {
              onSuccess: () => {
                undoToast({ title: t('bens-excluido'), description: deleting.name });
                setDeleting(null);
                update({ bem: null, acao: null });
              },
            },
          )
        }
      />
      <AssetFormDialog
        open={creating}
        onOpenChange={(o) => (!o ? update({ novo: null }) : undefined)}
        onSaved={(a) => update({ bem: a.id, novo: null })}
      />
    </div>
  );
}
