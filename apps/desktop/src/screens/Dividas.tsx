/**
 * 06 · DÍVIDAS: resumo, contratos (saldo, taxa, sistema, indexador, progresso, próxima
 * parcela, CET e custo real) e o detalhe do contrato escolhido, com saldo devedor,
 * juros × amortização, simulador de amortização e tabela de amortização.
 *
 * Estado na URL: `?contrato=<id>` escolhe o contrato; `?novo=contrato` abre o cadastro
 * (a barra de comando usa: "Cadastrar empréstimo ou financiamento"); `?arquivados=1` lista também os arquivados.
 */
import { useCallback, useRef, useState } from 'react';
import { useSearchParams } from 'react-router';

import { useDebtSummary, useLoans } from '@/api/hooks/debt';
import { useCountryFeatures } from '@/api/hooks/country';
import { screenMeta } from '@/app/screens';
import { Button, EmptyState, Icon, ScreenHeader, type ContextMenuEntry } from '@/design/components';
import { DebtSummaryCards, LoanTable } from '@/features/debts/DebtList';
import type { LoanView } from '@/api/generated/LoanView';
import { LoanDetail, useLoanActions, type LoanIntent } from '@/features/debts/LoanDetail';
import { LoanFormDialog } from '@/features/debts/LoanDialogs';
import { MarketRatesPanel } from '@/features/debts/MarketRatesPanel';
import { PayoffSimulator } from '@/features/planning/PayoffSimulator';
import { Checkbox, QueryState } from '@/features/transactions/ui/controls';
import { money, usePageContext } from '@/features/companion/pageContext';
import { t } from '@/i18n';
import { useBaseCurrency } from '@/state/currency';

const meta = screenMeta('dividas');

export default function Dividas() {
  const [params, setParams] = useSearchParams();
  const archived = params.get('arquivados') === '1';
  const summary = useDebtSummary();
  const loans = useLoans(archived);
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
  const creating = params.get('novo') === 'contrato';
  const actions = useLoanActions();
  const features = useCountryFeatures();
  const [intent, setIntent] = useState<LoanIntent | null>(null);
  const intentSeq = useRef(0);
  const list = loans.data ?? [];
  const selectedId = list.find((l) => l.id === params.get('contrato'))?.id ?? list[0]?.id ?? null;
  const selected = list.find((l) => l.id === selectedId) ?? null;

  // Contexto para o agente: totais das dívidas, contratos e o contrato aberto.
  const sum = summary.data;
  const base = useBaseCurrency();
  usePageContext({
    screen: 'dividas',
    title: `${meta.index} · ${meta.title}`,
    filters: { archived },
    selection: selected
      ? { loan_id: selected.id, name: selected.name, kind: selected.kind }
      : undefined,
    figures: sum
      ? {
          total_balance: money(sum.total_balance, base),
          monthly_commitment: money(sum.monthly_commitment, base),
          interest_remaining: money(sum.interest_remaining, base),
        }
      : undefined,
    items: list.map((l) => ({
      id: l.id,
      name: l.name,
      kind: l.kind,
      balance: money(l.balance, l.currency),
      annual_rate: l.annual_rate,
      remaining_installments: l.remaining_count,
      overdue: l.overdue_count,
    })),
  });

  // Menu de contexto do contrato: os botões do detalhe, a partir da lista.
  const select = (id: string) => {
    setIntent(null);
    update({ contrato: id });
  };
  const ask = (l: LoanView, kind: LoanIntent['kind']) => {
    update({ contrato: l.id });
    setIntent({ loanId: l.id, kind, seq: ++intentSeq.current });
  };
  const loanMenu = (l: LoanView): ContextMenuEntry[] => [
    { heading: l.name },
    { label: t('dividas-menu-abrir'), icon: 'arrowRight', onSelect: () => select(l.id) },
    { separator: true },
    { label: t('dividas-menu-amortizacao-extra'), icon: 'plus', onSelect: () => ask(l, 'extra') },
    {
      label: t('dividas-conciliar'),
      icon: 'check',
      disabled: !actions.canReconcile(l),
      disabledReason: t('dividas-conciliar-sem-conta'),
      onSelect: () => actions.doReconcile(l),
    },
    { separator: true },
    { label: t('dividas-menu-editar'), icon: 'sliders', onSelect: () => ask(l, 'edit') },
    {
      label: l.archived ? t('dividas-reativar') : t('dividas-arquivar'),
      icon: l.archived ? 'restore' : 'minus',
      onSelect: () => actions.toggleArchived(l),
    },
    {
      label: t('dividas-menu-excluir'),
      icon: 'x',
      danger: true,
      onSelect: () => ask(l, 'delete'),
    },
  ];

  return (
    <div className="flex flex-col gap-6">
      <ScreenHeader
        index={meta.index}
        title={meta.title}
        description={meta.summary}
        actions={
          <Button
            variant="primary"
            icon={<Icon name="plus" size={12} />}
            onClick={() => update({ novo: 'contrato' })}
          >
            {t('dividas-novo-contrato')}
          </Button>
        }
      />
      <QueryState
        loading={loans.isPending}
        error={loans.error ?? summary.error}
        onRetry={() => void loans.refetch()}
        label={t('dividas-carregando-contratos')}
      >
        {list.length === 0 && !archived ? (
          <>
            <EmptyState
              kicker={t('dividas-vazio-kicker')}
              title={t('dividas-vazio-titulo')}
              description={t('dividas-vazio-descricao')}
              actions={
                <Button variant="primary" onClick={() => update({ novo: 'contrato' })}>
                  {t('dividas-novo-contrato')}
                </Button>
              }
            />
            <PayoffSimulator hasLoans={false} />
            {features.br_credit_rates ? <MarketRatesPanel /> : null}
          </>
        ) : (
          <>
            {summary.data ? <DebtSummaryCards summary={summary.data} /> : null}
            <div className="flex flex-col gap-2">
              <div className="flex items-center justify-between gap-3">
                <h2 className="label-caps text-fg">{t('dividas-contratos')}</h2>
                <Checkbox
                  checked={archived}
                  onChange={(v) => update({ arquivados: v ? '1' : null })}
                  label={t('dividas-mostrar-arquivados')}
                />
              </div>
              <LoanTable
                loans={list}
                selectedId={selectedId}
                onSelect={select}
                rowMenu={loanMenu}
              />
            </div>
            {selected ? (
              <LoanDetail
                key={selected.id}
                loan={selected}
                intent={intent}
                onDeleted={() => update({ contrato: null })}
              />
            ) : null}
            <PayoffSimulator hasLoans={list.some((l) => !l.archived && l.balance > 0)} />
            {features.br_credit_rates ? <MarketRatesPanel /> : null}
          </>
        )}
      </QueryState>
      <LoanFormDialog
        open={creating}
        onOpenChange={(o) => (!o ? update({ novo: null }) : undefined)}
        onSaved={(l) => update({ contrato: l.id, novo: null })}
      />
    </div>
  );
}
