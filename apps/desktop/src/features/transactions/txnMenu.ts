/**
 * Menu de contexto da linha de lançamento (04 · Lançamentos): abrir, categorizar na
 * linha, marcar, status, conta, regra e apagar, as mesmas ações do diálogo, da barra de
 * lote e dos atalhos da tabela. Com duas ou mais linhas marcadas, o clique direito numa
 * delas abre o menu do lote (`bulkMenuItems`), com as ações da barra de lote (com duas,
 * ligar ou desligar a transferência). Os atalhos
 * dos dois menus valem com o foco na tabela, sem abrir o menu.
 */
import type { TxnKind } from '@/api/generated/TxnKind';
import type { TxnStatus } from '@/api/generated/TxnStatus';
import type { TxnView } from '@/api/generated/TxnView';
import type { ContextMenuEntry } from '@/design/components';
import { t } from '@/i18n';

import { TXN_STATUS } from './ui/labels';

export interface TxnMenuHandlers {
  open: () => void;
  /** Abre o seletor de categoria da linha (só receitas e despesas sem divisão). */
  categorize?: () => void;
  toggleMark: () => void;
  marked: boolean;
  setStatus: (status: TxnStatus) => void;
  openAccount: () => void;
  filterAccount: () => void;
  suggestRule: () => void;
  /** Escolhe o projeto do lançamento (obra, reforma, viagem…). */
  setProject: () => void;
  /** Marca este e os lançamentos com a mesma descrição (para categorizar de uma vez). */
  markSameDescription: () => void;
  /** Liga a outra conta da família (a perna que já existe lá ou uma nova). */
  linkTransfer?: () => void;
  /** Desfaz a ligação da transferência (as duas pernas ficam soltas). */
  unlinkTransfer?: () => void;
  /** Liga a saída ao cartão, como pagamento de fatura. */
  payCard?: () => void;
  /** Liga a saída a uma parcela de contrato (empréstimo, financiamento). */
  payLoan?: () => void;
  /** Vincula a uma operação (aporte na carteira, cotas capital, meta) ou mostra o vínculo. */
  linkOperation?: () => void;
  /** Detalha os itens da compra em Consumo (ou abre os que já existem). */
  itemize?: () => void;
  remove: () => void;
}

/** Ligações que se desfazem pelo menu: transferência, pagamento de fatura e aporte/resgate. */
export const UNLINKABLE: ReadonlySet<TxnKind> = new Set<TxnKind>([
  'transfer',
  'card_payment',
  'investment',
]);

/** Receita, despesa ou transferência solta (sem a outra perna, divisão nem parcelamento). */
export const isLooseTxn = (t: TxnView) =>
  (t.kind === 'income' || t.kind === 'expense' || t.kind === 'transfer') &&
  !t.transfer_id &&
  t.splits.length === 0 &&
  !t.installment_plan_id;

export function txnMenuItems(txn: TxnView, h: TxnMenuHandlers): ContextMenuEntry[] {
  const isLeg = Boolean(txn.transfer_id);
  const reconciled = txn.status === 'reconciled';
  const status = (s: TxnStatus): ContextMenuEntry =>
    txn.status !== s && {
      label: t('lancamentos-marcar-como', { status: TXN_STATUS[s].toLowerCase() }),
      icon: 'check',
      onSelect: () => h.setStatus(s),
    };
  return [
    { heading: txn.description || t('lancamentos-sem-descricao-parenteses') },
    {
      label: t('lancamentos-menu-abrir'),
      icon: 'arrowRight',
      shortcut: ['enter'],
      onSelect: h.open,
    },
    h.categorize && {
      label: t('lancamentos-menu-categorizar'),
      icon: 'reticle',
      shortcut: ['C'],
      onSelect: h.categorize,
    },
    txn.splits.length > 0 && {
      label: t('lancamentos-menu-ver-divisao'),
      icon: 'sort',
      onSelect: h.open,
    },
    {
      label: h.marked ? t('lancamentos-menu-desmarcar') : t('lancamentos-menu-marcar-lote'),
      icon: 'check',
      shortcut: ['Espaço'],
      onSelect: h.toggleMark,
    },
    {
      label: t('lancamentos-menu-mesma-descricao'),
      icon: 'check',
      onSelect: h.markSameDescription,
    },
    {
      label: txn.project_name
        ? t('lancamentos-menu-projeto', { projeto: txn.project_name })
        : t('lancamentos-menu-por-projeto'),
      icon: 'sliders',
      onSelect: h.setProject,
    },
    { separator: true },
    h.linkTransfer && {
      label:
        txn.amount < 0
          ? t('lancamentos-menu-transferencia-para')
          : t('lancamentos-menu-transferencia-de'),
      icon: 'arrowRight',
      onSelect: h.linkTransfer,
    },
    h.unlinkTransfer && {
      label: txn.counterpart_account_name
        ? t('lancamentos-menu-desligar-de', { conta: txn.counterpart_account_name })
        : t('lancamentos-desligar-transferencia'),
      icon: 'minus',
      onSelect: h.unlinkTransfer,
    },
    h.payCard && {
      label: t('lancamentos-menu-pagamento-fatura'),
      icon: 'arrowRight',
      onSelect: h.payCard,
    },
    h.payLoan && {
      label: txn.loan_id
        ? t('lancamentos-menu-parcela', {
            numero: txn.loan_installment_number ?? '',
            contrato: txn.loan_name ?? t('lancamentos-contrato'),
          })
        : t('lancamentos-menu-pagamento-parcela'),
      icon: 'arrowRight',
      onSelect: h.payLoan,
    },
    h.linkOperation && {
      label: txn.operation
        ? t('lancamentos-menu-vinculo', { operacao: txn.operation.label })
        : t('lancamentos-menu-vincular'),
      icon: 'arrowRight',
      onSelect: h.linkOperation,
    },
    h.itemize && {
      label: txn.receipt_id ? t('alimentacao-ver-itens') : t('alimentacao-detalhar-itens'),
      icon: 'plus',
      onSelect: h.itemize,
    },
    { separator: true },
    status('cleared'),
    status('planned'),
    { separator: true },
    {
      label: t('lancamentos-menu-abrir-conta', { conta: txn.account_name }),
      icon: 'arrowRight',
      onSelect: h.openAccount,
    },
    { label: t('lancamentos-menu-so-conta'), icon: 'search', onSelect: h.filterAccount },
    !isLeg && {
      label: t('lancamentos-criar-regra-deste'),
      icon: 'reticle',
      onSelect: h.suggestRule,
    },
    { separator: true },
    {
      label: t('lancamentos-menu-apagar'),
      icon: 'x',
      danger: true,
      shortcut: ['mod', 'Backspace'],
      onSelect: h.remove,
      ...(reconciled
        ? { disabled: true, disabledReason: t('lancamentos-conciliado-nao-apaga') }
        : {}),
    },
  ];
}

export interface BulkMenuHandlers {
  categorize: () => void;
  setStatus: (status: TxnStatus) => void;
  member: () => void;
  tags: () => void;
  project: () => void;
  markSame: () => void;
  /** Marca todos os `filterTotal` lançamentos do filtro (quando são mais que os marcados). */
  markAll?: () => void;
  filterTotal?: number;
  suggestRule?: () => void;
  /** Duas marcadas, uma saída e uma entrada em contas diferentes: ligar como transferência. */
  linkPair?: () => void;
  /** Duas marcadas que não formam o par: por quê (o item aparece desabilitado). */
  pairIssue?: string;
  /** As duas marcadas são as pernas da mesma transferência: desligar. */
  unlinkPair?: () => void;
  /** Vincula os marcados a uma operação (com confirmação). */
  linkOperation?: () => void;
  clear: () => void;
  remove: () => void;
}

/** Menu do lote: as ações da barra de lote sobre os `n` lançamentos marcados. */
export function bulkMenuItems(n: number, h: BulkMenuHandlers): ContextMenuEntry[] {
  return [
    { heading: t('lancamentos-menu-n-marcados', { n }) },
    {
      label: t('lancamentos-menu-categorizar-marcados'),
      icon: 'reticle',
      shortcut: ['C'],
      onSelect: h.categorize,
    },
    { label: t('lancamentos-menu-membro'), icon: 'sliders', onSelect: h.member },
    { label: t('lancamentos-menu-tags'), icon: 'sliders', onSelect: h.tags },
    { label: t('lancamentos-menu-por-projeto'), icon: 'sliders', onSelect: h.project },
    (h.linkPair != null || Boolean(h.pairIssue)) && {
      label: t('lancamentos-menu-ligar-par'),
      icon: 'arrowRight',
      onSelect: h.linkPair ?? (() => undefined),
      ...(h.linkPair ? {} : { disabled: true, disabledReason: h.pairIssue }),
    },
    h.unlinkPair && {
      label: t('lancamentos-desligar-transferencia'),
      icon: 'minus',
      onSelect: h.unlinkPair,
    },
    h.linkOperation && {
      label: t('lancamentos-menu-vincular'),
      icon: 'arrowRight',
      onSelect: h.linkOperation,
    },
    { separator: true },
    ...(['planned', 'cleared', 'reconciled'] as TxnStatus[]).map((s): ContextMenuEntry => ({
      label: t('lancamentos-marcar-como', { status: TXN_STATUS[s].toLowerCase() }),
      icon: 'check',
      onSelect: () => h.setStatus(s),
    })),
    { separator: true },
    { label: t('lancamentos-menu-mesma-descricao'), icon: 'check', onSelect: h.markSame },
    h.markAll &&
      h.filterTotal !== undefined &&
      h.filterTotal > n && {
        label: t('lancamentos-menu-marcar-todos', { n: h.filterTotal }),
        icon: 'check',
        onSelect: h.markAll,
      },
    h.suggestRule && {
      label: t('lancamentos-criar-regra'),
      icon: 'reticle',
      onSelect: h.suggestRule,
    },
    { label: t('controles-limpar-selecao'), icon: 'minus', shortcut: ['esc'], onSelect: h.clear },
    { separator: true },
    {
      label: t('lancamentos-menu-apagar-n', { n }),
      icon: 'x',
      danger: true,
      shortcut: ['mod', 'Backspace'],
      onSelect: h.remove,
    },
  ];
}
