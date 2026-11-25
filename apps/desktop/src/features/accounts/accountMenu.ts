/**
 * Menu de contexto do card de conta (03 · Contas): os mesmos atalhos dos botões do card,
 * do detalhe da conta e do cadastro, num lugar só.
 */
import type { AccountView } from '@/api/generated/AccountView';
import type { ContextMenuEntry } from '@/design/components';
import type { TxnDialogMode } from '@/features/transactions/TxnDialog';
import { t } from '@/i18n';
import { baseCurrency } from '@/lib/format';

export interface AccountMenuHandlers {
  /** Abre o detalhe da conta, numa aba (`historico`, `faturas`, `conciliacao`…). */
  open: (tab?: string) => void;
  newTxn: (mode: TxnDialogMode) => void;
  viewTxns: () => void;
  importStatement: () => void;
  edit: () => void;
  toggleArchived: () => void;
  /** Opens the closing dialog (account closed at the bank). */
  close: () => void;
  /** Reopens a closed account. */
  reopen: () => void;
}

export function accountMenuItems(a: AccountView, h: AccountMenuHandlers): ContextMenuEntry[] {
  const isCard = a.kind === 'credit_card';
  return [
    { heading: a.name },
    { label: t('contas-menu-abrir-detalhes'), icon: 'arrowRight', onSelect: () => h.open() },
    { separator: true },
    {
      label: isCard ? t('contas-menu-nova-compra') : t('contas-menu-novo-lancamento'),
      icon: 'plus',
      onSelect: () => h.newTxn('expense'),
    },
    isCard && {
      label: t('contas-menu-compra-parcelada'),
      icon: 'plus',
      onSelect: () => h.newTxn('installments'),
    },
    !isCard && {
      label: t('contas-menu-transferir'),
      icon: 'arrowRight',
      onSelect: () => h.newTxn('transfer'),
    },
    { label: t('contas-menu-ver-lancamentos'), icon: 'search', onSelect: h.viewTxns },
    { label: t('contas-menu-importar'), icon: 'clip', onSelect: h.importStatement },
    { separator: true },
    isCard && { label: t('contas-faturas'), icon: 'history', onSelect: () => h.open('faturas') },
    isCard && {
      label: t('contas-parcelamentos'),
      icon: 'history',
      onSelect: () => h.open('parcelamentos'),
    },
    { label: t('contas-historico-saldo'), icon: 'history', onSelect: () => h.open('historico') },
    { label: t('contas-menu-conciliar'), icon: 'check', onSelect: () => h.open('conciliacao') },
    a.currency !== baseCurrency() && {
      label: t('contas-menu-cambio', { moeda: a.currency, base: baseCurrency() }),
      icon: 'refresh',
      onSelect: () => h.open('cambio'),
    },
    { separator: true },
    { label: t('contas-menu-editar'), icon: 'sliders', onSelect: h.edit },
    a.closed_on
      ? { label: t('contas-reabrir'), icon: 'restore', onSelect: h.reopen }
      : {
          label: a.archived ? t('contas-reativar') : t('contas-arquivar'),
          icon: a.archived ? 'restore' : 'minus',
          onSelect: h.toggleArchived,
        },
    !a.closed_on && { label: t('contas-menu-encerrar'), icon: 'minus', onSelect: h.close },
  ];
}
