/**
 * Detalhe de um bem: medidas (valor atual com a origem, custo, ganho nominal e real,
 * valorização anualizada, patrimônio do bem e LTV), como o valor foi calculado, as figuras,
 * o financiamento ligado (com link para 07 · Dívidas), as benfeitorias e as avaliações,
 * com as ações de editar, avaliar, consultar a FIPE, vender, arquivar e excluir.
 */
import { useState } from 'react';
import { Link } from 'react-router';

import { useCountryFeatures } from '@/api/hooks/country';
import { useMembers } from '@/api/hooks/household';
import {
  useDeleteImprovement,
  useDeletePhysical,
  useDeleteValuation,
  usePhysicalHistory,
  useRefreshFipe,
  useUnlinkLoan,
  useUpdatePhysical,
} from '@/api/hooks/physical';
import { useUndoToast } from '@/api/hooks/undoToast';
import type { PhysicalAssetView } from '@/api/generated/PhysicalAssetView';
import type { PhysicalImprovement } from '@/api/generated/PhysicalImprovement';
import type { PhysicalLinkedLoan } from '@/api/generated/PhysicalLinkedLoan';
import type { PhysicalValuation } from '@/api/generated/PhysicalValuation';
import {
  Button,
  Icon,
  InstrumentCard,
  KeyValue,
  Measure,
  Money,
  TechTable,
  toast,
} from '@/design/components';
import { KIND_LABEL as LOAN_KIND_LABEL } from '@/features/debts/labels';
import { Pct } from '@/features/portfolio/components/Pct';
import { Notice } from '@/features/portfolio/components/States';
import { ConfirmDialog, QueryState } from '@/features/transactions/ui/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { txnLink } from '@/lib/txnLink';
import { formatDate } from '@/lib/format';

import { BASIS_LABEL, CATEGORY_LABEL, COST_KIND_LABEL, KIND_LABEL, SOURCE_LABEL } from './labels';
import {
  AssetFormDialog,
  ImprovementDialog,
  LinkLoanDialog,
  SaleDialog,
  ValuationDialog,
} from './PhysicalDialogs';
import { CapitalGainCard } from './CapitalGainCard';
import { IptuPanel } from './IptuPanel';
import { EquityFigure, ValueFigure } from './PhysicalFigures';
import { acquisitionTotal, methodText, ownersText } from './transform';

export type DetailDialog = 'edit' | 'improvement' | 'valuation' | 'sale' | 'loan' | null;

export function PhysicalDetail({
  asset,
  dialog,
  onDialog,
  onDeleted,
}: {
  asset: PhysicalAssetView;
  dialog: DetailDialog;
  onDialog: (d: DetailDialog) => void;
  onDeleted: () => void;
}) {
  const history = usePhysicalHistory(asset.id);
  const features = useCountryFeatures();
  const members = useMembers().data ?? [];
  const update = useUpdatePhysical();
  const remove = useDeletePhysical();
  const delImprovement = useDeleteImprovement();
  const delValuation = useDeleteValuation();
  const unlink = useUnlinkLoan();
  const refreshFipe = useRefreshFipe();
  const undoToast = useUndoToast();
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [editImprovement, setEditImprovement] = useState<PhysicalImprovement | null>(null);
  const f = asset.figures;
  const hasFipe = Boolean(asset.fipe_brand_code && asset.fipe_model_code && asset.fipe_year_code);
  const points = history.data ?? [];

  const fail = (title: string) => (e: Error) =>
    toast({ tone: 'error', title, description: e.message });

  return (
    <section aria-label={t('bens-bem-nome', { nome: asset.name })} className="flex flex-col gap-5">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex min-w-0 flex-col gap-1">
          <h2 className="text-[18px] font-medium text-fg">{asset.name}</h2>
          <p className="text-[13px] text-dim">
            {[
              KIND_LABEL[asset.kind],
              ownersText(asset.member_ids, members),
              t('bens-comprado-em', { data: formatDate(asset.purchase_date) }),
              asset.address,
              f.sold && asset.sale_date
                ? t('bens-vendido-em', { data: formatDate(asset.sale_date) })
                : null,
              asset.archived ? t('bens-arquivado') : null,
              asset.include_in_net_worth ? null : t('bens-fora-patrimonio'),
            ]
              .filter(Boolean)
              .join(' · ')}
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Button variant="ghost" onClick={() => onDialog('edit')}>
            {t('comum-editar')}
          </Button>
          <Button variant="ghost" onClick={() => onDialog('valuation')}>
            {t('bens-registrar-avaliacao')}
          </Button>
          {hasFipe ? (
            <Button
              variant="ghost"
              loading={refreshFipe.isPending}
              onClick={() =>
                refreshFipe.mutate(
                  { id: asset.id },
                  {
                    onSuccess: (v) =>
                      undoToast({
                        tone: 'success',
                        title: t('bens-fipe-consultada'),
                        description: t('bens-fipe-valor-atualizado', { nome: v.name }),
                      }),
                    onError: fail(t('bens-fipe-falhou')),
                  },
                )
              }
            >
              {t('bens-consultar-fipe')}
            </Button>
          ) : null}
          {!f.sold ? (
            <Button variant="ghost" onClick={() => onDialog('sale')}>
              {t('bens-vender')}
            </Button>
          ) : null}
          <Button
            variant="ghost"
            loading={update.isPending}
            onClick={() =>
              update.mutate(
                { id: asset.id, archived: !asset.archived },
                {
                  onSuccess: () =>
                    undoToast({
                      title: asset.archived ? t('bens-reativado') : t('bens-arquivado-toast'),
                    }),
                },
              )
            }
          >
            {asset.archived ? t('bens-reativar') : t('bens-arquivar')}
          </Button>
          <Button variant="ghost" onClick={() => setConfirmDelete(true)}>
            {t('bens-excluir')}
          </Button>
          <Button
            variant="primary"
            icon={<Icon name="plus" size={12} />}
            onClick={() => {
              setEditImprovement(null);
              onDialog('improvement');
            }}
          >
            {t('bens-benfeitoria')}
          </Button>
        </div>
      </div>

      <div className="grid gap-4 @min-[530px]:grid-cols-2 @min-[1300px]:grid-cols-4">
        <InstrumentCard index="07.1" title={t('bens-valor-atual')} headingLevel={3}>
          <Measure
            label={
              f.sold
                ? t('bens-vendido-maiusc')
                : t('bens-por-base', { base: BASIS_LABEL[f.value_basis] })
            }
            size="md"
            value={<Money cents={f.sold ? (f.sale_net ?? 0) : f.value} estimated={f.estimated} />}
            origin={{
              source: f.source,
              date: f.anchor_date,
              status: f.estimated ? 'estimado' : 'oficial',
              note: methodText(asset),
            }}
            hint={f.sold ? t('bens-liquido-venda') : methodText(asset)}
          />
        </InstrumentCard>
        <InstrumentCard index="07.2" title={t('bens-custo')} headingLevel={3}>
          <KeyValue
            items={[
              { label: t('bens-compra'), value: <Money cents={asset.purchase_price} /> },
              {
                label: t('bens-custos-aquisicao'),
                value: <Money cents={acquisitionTotal(asset)} />,
              },
              { label: t('bens-benfeitorias'), value: <Money cents={f.capital_improvements} /> },
              { label: t('bens-custo-total'), value: <Money cents={f.cost_basis} /> },
              ...(f.maintenance > 0
                ? [{ label: t('bens-manutencao-fora'), value: <Money cents={f.maintenance} /> }]
                : []),
            ]}
          />
        </InstrumentCard>
        <InstrumentCard
          index="07.3"
          title={f.value >= f.cost_basis || f.sold ? t('bens-valorizacao') : t('bens-depreciacao')}
          headingLevel={3}
        >
          <KeyValue
            items={[
              {
                label: f.sold ? t('bens-ganho-realizado') : t('bens-ganho-nominal'),
                value: <Money cents={f.nominal_gain} tone="auto" signed />,
              },
              {
                label: t('bens-sobre-custo'),
                value: <Pct value={f.nominal_gain_pct} signed decimals={1} />,
              },
              {
                label: t('bens-ganho-real'),
                value: f.real_gain != null ? <Money cents={f.real_gain} tone="auto" signed /> : '—',
              },
              { label: t('bens-ao-ano'), value: <Pct value={f.annualized} signed decimals={2} /> },
              {
                label: t('bens-ao-ano-real'),
                value: <Pct value={f.real_annualized} signed decimals={2} />,
              },
            ]}
          />
        </InstrumentCard>
        <InstrumentCard index="07.4" title={t('bens-patrimonio-do-bem')} headingLevel={3}>
          <Measure
            label={t('bens-valor-menos-saldo')}
            size="md"
            value={<Money cents={f.equity} estimated={f.estimated} />}
            hint={
              asset.loans.length > 0
                ? withSlots(t('bens-saldo-ltv', { saldo: slot(0), ltv: slot(1) }), [
                    <Money key="saldo" cents={f.loan_balance} />,
                    <Pct key="ltv" value={f.ltv} decimals={1} />,
                  ])
                : t('bens-sem-financiamento')
            }
          />
        </InstrumentCard>
      </div>

      <Notice
        tone={f.estimated ? 'warning' : 'info'}
        title={f.estimated ? t('bens-valor-estimado-titulo') : t('bens-valor-medido')}
      >
        {f.formula}
      </Notice>

      {features.br_income_tax ? <CapitalGainCard asset={asset} /> : null}

      <QueryState
        loading={history.isPending}
        error={history.error}
        onRetry={() => void history.refetch()}
        label={t('bens-calculando-historico')}
      >
        {points.length > 0 ? (
          <div className="grid gap-6 @min-[1040px]:grid-cols-2">
            <ValueFigure index="07.5" asset={asset} history={points} />
            {asset.loans.length > 0 ? (
              <EquityFigure index="07.6" asset={asset} history={points} />
            ) : null}
          </div>
        ) : null}
      </QueryState>

      <InstrumentCard
        index="07.7"
        title={t('bens-financiamento')}
        actions={
          <Button variant="ghost" size="sm" onClick={() => onDialog('loan')}>
            {t('bens-ligar-financiamento')}
          </Button>
        }
      >
        <TechTable<PhysicalLinkedLoan>
          label={t('bens-financiamentos-de', { nome: asset.name })}
          prefsId={`Financiamentos de ${asset.name}`}
          dense
          rows={asset.loans}
          getRowId={(l) => l.loan_id}
          empty={t('bens-financiamentos-vazio')}
          columns={[
            {
              key: 'name',
              header: t('bens-contrato'),
              cell: (l) => (
                <Link
                  to={`/dividas?contrato=${encodeURIComponent(l.loan_id)}`}
                  className="text-accent underline-offset-2 hover:underline"
                >
                  {l.name}
                </Link>
              ),
            },
            { key: 'kind', header: t('bens-tipo'), cell: (l) => LOAN_KIND_LABEL[l.kind] },
            {
              key: 'principal',
              header: t('bens-financiado'),
              numeric: true,
              cell: (l) => <Money cents={l.principal} />,
            },
            {
              key: 'balance',
              header: t('bens-saldo-devedor'),
              numeric: true,
              cell: (l) => <Money cents={l.balance} />,
            },
            {
              key: 'actions',
              header: '',
              cell: (l) => (
                <Button
                  variant="ghost"
                  size="sm"
                  aria-label={t('bens-desligar-nome', { nome: l.name })}
                  onClick={() =>
                    unlink.mutate(
                      { asset_id: asset.id, loan_id: l.loan_id },
                      {
                        onSuccess: () =>
                          undoToast({
                            title: t('bens-financiamento-desligado'),
                            description: l.name,
                          }),
                        onError: fail(t('bens-nao-desligou')),
                      },
                    )
                  }
                >
                  {t('bens-desligar')}
                </Button>
              ),
            },
          ]}
        />
      </InstrumentCard>

      <InstrumentCard
        index="07.8"
        title={t('bens-benfeitorias-titulo')}
        footer={[
          { label: t('bens-entram-custo'), value: t('bens-entram-custo-valor') },
          { label: t('bens-fora-custo-rotulo'), value: t('bens-fora-custo-valor') },
        ]}
      >
        <TechTable<PhysicalImprovement>
          label={t('bens-benfeitorias-de', { nome: asset.name })}
          prefsId={`Benfeitorias de ${asset.name}`}
          dense
          rows={asset.improvements}
          getRowId={(i) => i.id}
          empty={t('bens-benfeitorias-vazio')}
          onActivate={(i) => {
            setEditImprovement(i);
            onDialog('improvement');
          }}
          columns={[
            {
              key: 'date',
              header: t('bens-data'),
              sortValue: (i) => i.date,
              cell: (i) => formatDate(i.date),
            },
            {
              key: 'description',
              header: t('bens-descricao'),
              cell: (i) => (
                <span className="flex flex-col">
                  <span>{i.description}</span>
                  {i.notes ? <span className="text-[11.5px] text-dim">{i.notes}</span> : null}
                </span>
              ),
            },
            {
              key: 'category',
              header: t('bens-tipo'),
              cell: (i) => (
                <span className={i.capital ? '' : 'text-dim'}>
                  {CATEGORY_LABEL[i.category]}
                  {i.capital ? '' : ` · ${t('bens-fora-custo')}`}
                </span>
              ),
            },
            {
              key: 'project',
              header: t('bens-projeto'),
              sortValue: (i) => i.project_name,
              cell: (i) =>
                i.project_id ? (
                  <span className="flex flex-col">
                    <span className="truncate">{i.project_name}</span>
                    <Link
                      to={txnLink({ project_ids: [i.project_id] })}
                      className="text-[11.5px] text-accent hover:underline"
                    >
                      {i.linked_count === 0
                        ? t('bens-sem-lancamentos-estimado')
                        : t('bens-n-lancamentos', { n: i.linked_count })}
                    </Link>
                  </span>
                ) : (
                  <span className="text-dim">—</span>
                ),
            },
            {
              key: 'amount',
              header: t('bens-valor'),
              numeric: true,
              sortValue: (i) => i.amount,
              cell: (i) => <Money cents={i.amount} />,
            },
            {
              key: 'actions',
              header: '',
              cell: (i) => (
                <span className="flex justify-end gap-1">
                  <Button
                    variant="ghost"
                    size="sm"
                    aria-label={t('bens-editar-nome', { nome: i.description })}
                    onClick={() => {
                      setEditImprovement(i);
                      onDialog('improvement');
                    }}
                  >
                    {t('comum-editar')}
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    aria-label={t('bens-excluir-nome', { nome: i.description })}
                    onClick={() =>
                      delImprovement.mutate(
                        { id: i.id },
                        {
                          onSuccess: () =>
                            undoToast({
                              title: t('bens-benfeitoria-excluida'),
                              description: i.description,
                            }),
                          onError: fail(t('bens-nao-excluiu')),
                        },
                      )
                    }
                  >
                    {t('bens-excluir')}
                  </Button>
                </span>
              ),
            },
          ]}
        />
      </InstrumentCard>

      <InstrumentCard index="07.9" title={t('bens-avaliacoes')}>
        <TechTable<PhysicalValuation>
          label={t('bens-avaliacoes-de', { nome: asset.name })}
          prefsId={`Avaliações de ${asset.name}`}
          dense
          rows={[...asset.valuations].reverse()}
          getRowId={(v) => v.id}
          empty={t('bens-avaliacoes-vazio')}
          columns={[
            { key: 'date', header: t('bens-data'), cell: (v) => formatDate(v.date) },
            { key: 'source', header: t('bens-fonte'), cell: (v) => SOURCE_LABEL[v.source] },
            {
              key: 'value',
              header: t('bens-valor'),
              numeric: true,
              cell: (v) => <Money cents={v.value} />,
            },
            {
              key: 'notes',
              header: t('bens-observacoes'),
              cell: (v) => <span className="text-dim">{v.notes ?? ''}</span>,
            },
            {
              key: 'actions',
              header: '',
              cell: (v) => (
                <Button
                  variant="ghost"
                  size="sm"
                  aria-label={t('bens-excluir-avaliacao-de', { data: formatDate(v.date) })}
                  onClick={() =>
                    delValuation.mutate(
                      { id: v.id },
                      {
                        onSuccess: () => undoToast({ title: t('bens-avaliacao-excluida') }),
                        onError: fail(t('bens-nao-excluiu')),
                      },
                    )
                  }
                >
                  {t('bens-excluir')}
                </Button>
              ),
            },
          ]}
        />
      </InstrumentCard>

      <IptuPanel asset={asset} />

      {asset.acquisition_costs.length > 0 ? (
        <p className="text-[12.5px] text-dim">
          {t('bens-custos-aquisicao-prefixo')}{' '}
          {asset.acquisition_costs.map((c, i) => (
            <span key={c.id}>
              {i > 0 ? ' · ' : ''}
              {c.description ?? COST_KIND_LABEL[c.kind]} <Money cents={c.amount} />
            </span>
          ))}
        </p>
      ) : null}
      {asset.notes ? (
        <p className="text-[12.5px] text-dim">
          {t('bens-observacoes-texto', { notas: asset.notes })}
        </p>
      ) : null}

      <AssetFormDialog
        open={dialog === 'edit'}
        onOpenChange={(o) => onDialog(o ? 'edit' : null)}
        asset={asset}
      />
      <ImprovementDialog
        open={dialog === 'improvement'}
        onOpenChange={(o) => onDialog(o ? 'improvement' : null)}
        asset={asset}
        improvement={editImprovement}
      />
      <ValuationDialog
        open={dialog === 'valuation'}
        onOpenChange={(o) => onDialog(o ? 'valuation' : null)}
        asset={asset}
      />
      <SaleDialog
        open={dialog === 'sale'}
        onOpenChange={(o) => onDialog(o ? 'sale' : null)}
        asset={asset}
      />
      <LinkLoanDialog
        open={dialog === 'loan'}
        onOpenChange={(o) => onDialog(o ? 'loan' : null)}
        asset={asset}
      />
      <ConfirmDialog
        open={confirmDelete}
        onOpenChange={setConfirmDelete}
        title={t('bens-excluir-titulo', { nome: asset.name })}
        description={t('bens-excluir-descricao-detalhe')}
        confirmLabel={t('bens-excluir-bem')}
        pending={remove.isPending}
        error={remove.error}
        onConfirm={() =>
          remove.mutate(
            { id: asset.id },
            {
              onSuccess: () => {
                undoToast({ tone: 'success', title: t('bens-excluido'), description: asset.name });
                setConfirmDelete(false);
                onDeleted();
              },
            },
          )
        }
      />
    </section>
  );
}
