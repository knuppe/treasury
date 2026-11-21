import { describe, expect, it } from 'vitest';

import type { ChangeItem } from '@/api/generated/ChangeItem';

import {
  describeChanges,
  fieldLabel,
  formatFieldValue,
  friendlyRow,
  referencedIds,
  rowTitle,
  type NameBook,
  type RefKind,
} from './changeText';

const NAMES: Record<string, string> = {
  'category:cat-mercado': 'Alimentação › Mercado',
  'category:cat-outros': 'Outros',
  'category:cat-app': 'Transporte › Aplicativos',
  'account:acc-nubank': 'Cartão Nubank',
  'account:acc-xp': 'XP Investimentos',
  'member:mem-ana': 'Ana',
  'asset:ast-petr4': 'PETR4',
  'tag:tag-1': '#viagem',
};

const TXNS = {
  'txn-1': { description: 'UBER *TRIP', date: '2026-10-08', amount: -3_270, currency: 'BRL' },
  'txn-2': { description: 'UBER *TRIP HELP', date: '2026-10-09', amount: -1_990, currency: 'BRL' },
  'txn-3': { description: '99 POP', date: '2026-10-10', amount: -2_450, currency: 'BRL' },
};

const book: NameBook = {
  name: (kind: RefKind, id: string) => NAMES[`${kind}:${id}`],
  txn: (id: string) => TXNS[id as keyof typeof TXNS],
};

/** Alteração como o núcleo grava: só o id, o que mudou e a data técnica. */
function setCategory(seq: number, id: string, from: string | null, to: string): ChangeItem {
  return {
    seq,
    table_name: 'txn',
    row_id: id,
    op: 'update',
    before: { id, category_id: from, updated_at: '2026-10-08T12:00:00Z' },
    after: { id, category_id: to, updated_at: '2026-10-14T09:12:00Z' },
  };
}

describe('leitura amigável das alterações', () => {
  it('troca ids por nomes e formata dinheiro, datas, enums e booleanos', () => {
    const row = { currency: 'USD' };
    expect(formatFieldValue('txn', 'category_id', 'cat-mercado', null, book)).toBe(
      'Alimentação › Mercado',
    );
    expect(formatFieldValue('txn', 'category_id', null, null, book)).toBe('Sem categoria');
    expect(formatFieldValue('txn', 'account_id', 'acc-sumiu', null, book)).toBe(
      'conta (não encontrado)',
    );
    expect(formatFieldValue('txn', 'amount', -18_990, null, book)).toMatch(/R\$\s?189,90/);
    expect(formatFieldValue('txn', 'amount', 10_000, row, book)).toMatch(/USD\s?100,00/);
    expect(formatFieldValue('txn', 'date', '2026-10-08', null, book)).toBe('08/10/2026');
    expect(formatFieldValue('txn', 'kind', 'expense', null, book)).toBe('Despesa');
    expect(formatFieldValue('txn', 'status', 'planned', null, book)).toBe('Previsto');
    expect(formatFieldValue('account', 'kind', 'credit_card', null, book)).toBe(
      'Cartão de crédito',
    );
    expect(formatFieldValue('trade', 'side', 'buy', null, book)).toBe('Compra');
    expect(formatFieldValue('rule', 'active', 1, null, book)).toBe('sim');
    expect(formatFieldValue('txn', 'transfer_id', 'txn-3', null, book)).toBe('“99 POP”');
    expect(fieldLabel('category_id')).toBe('Categoria');
    expect(fieldLabel('to_account_id')).toBe('Conta de destino');
  });

  it('regras: condições e ações em texto, vindas como JSON do SQLite', () => {
    const conditions = JSON.stringify([{ field: 'description', op: 'contains', value: 'UBER' }]);
    expect(formatFieldValue('rule', 'conditions', conditions, { match_all: 1 }, book)).toBe(
      'Descrição contém “UBER”',
    );
    expect(
      formatFieldValue(
        'rule',
        'actions',
        JSON.stringify({ category_id: 'cat-app', tags: ['corrida'] }),
        null,
        book,
      ),
    ).toBe('categoria Transporte › Aplicativos · etiquetas #corrida');
  });

  it('título da linha: lançamento achado pelo id quando a alteração não traz a descrição', () => {
    const t = rowTitle(setCategory(1, 'txn-1', null, 'cat-app'), book);
    expect(t.name).toBe('UBER *TRIP');
    expect(t.detail).toBe('08/10/2026');
    expect(t.money).toMatch(/32,70/);
    const trade: ChangeItem = {
      seq: 1,
      table_name: 'trade',
      row_id: 'tr-1',
      op: 'insert',
      before: null,
      after: {
        id: 'tr-1',
        asset_id: 'ast-petr4',
        account_id: 'acc-xp',
        side: 'buy',
        date: '2026-10-10',
        quantity: '100',
        price: '38.50',
        fees: 490,
        currency: 'BRL',
      },
    };
    expect(rowTitle(trade, book)).toMatchObject({
      name: 'Compra de PETR4',
      detail: '10/10/2026',
    });
    expect(rowTitle(trade, book).money).toMatch(/^100 × R\$\s?38,50$/);
    const row = friendlyRow(trade, book);
    expect(row.opLabel).toBe('Criação');
    // Ids técnicos e a moeda (já no valor) ficam de fora; nomes no lugar dos ids.
    expect(row.fields.map((f) => [f.label, f.after])).toEqual([
      ['Ativo', 'PETR4'],
      ['Conta', 'XP Investimentos'],
      ['Operação', 'Compra'],
      ['Data', '10/10/2026'],
      ['Quantidade', '100'],
      ['Preço', expect.stringMatching(/R\$\s?38,50/)],
      ['Custos', expect.stringMatching(/R\$\s?4,90/)],
    ]);
    expect(row.fields.find((f) => f.field === 'fees')?.sensitive).toBe(true);
  });

  it('alteração: só os campos que mudam, sem a data técnica', () => {
    const row = friendlyRow(setCategory(1, 'txn-2', 'cat-outros', 'cat-app'), book);
    expect(row.fields).toEqual([
      {
        field: 'category_id',
        label: 'Categoria',
        before: 'Outros',
        after: 'Transporte › Aplicativos',
        change: 'changed',
        sensitive: false,
      },
    ]);
  });

  it('resume em frases: mesma categoria para vários lançamentos', () => {
    expect(
      describeChanges(
        [
          setCategory(1, 'txn-1', 'cat-outros', 'cat-app'),
          setCategory(2, 'txn-2', 'cat-outros', 'cat-app'),
          setCategory(3, 'txn-3', 'cat-outros', 'cat-app'),
        ],
        book,
      ),
    ).toEqual(['Altera a categoria de 3 lançamentos: Outros → Transporte › Aplicativos']);
    expect(
      describeChanges(
        [
          setCategory(1, 'txn-1', null, 'cat-app'),
          setCategory(2, 'txn-2', 'cat-outros', 'cat-app'),
        ],
        book,
      ),
    ).toEqual(['Altera a categoria de 2 lançamentos para Transporte › Aplicativos']);
    expect(describeChanges([setCategory(1, 'txn-1', null, 'cat-app')], book)).toEqual([
      'Altera a categoria de “UBER *TRIP”: Sem categoria → Transporte › Aplicativos',
    ]);
    const rule: ChangeItem = {
      seq: 2,
      table_name: 'rule',
      row_id: 'r-1',
      op: 'insert',
      before: null,
      after: { id: 'r-1', name: 'Corridas de aplicativo' },
    };
    const removed: ChangeItem = {
      seq: 3,
      table_name: 'txn',
      row_id: 'txn-9',
      op: 'delete',
      before: { id: 'txn-9', description: 'Duplicado', amount: -100, date: '2026-10-01' },
      after: null,
    };
    expect(describeChanges([rule, removed], book)).toEqual([
      'Cria 1 regra: “Corridas de aplicativo”',
      'Apaga 1 lançamento: “Duplicado”',
    ]);
  });

  it('lista os ids citados para buscar só o que falta', () => {
    const refs = referencedIds([
      setCategory(1, 'txn-1', 'cat-outros', 'cat-app'),
      {
        seq: 2,
        table_name: 'txn_tag',
        row_id: 'txn-1:tag-1',
        op: 'insert',
        before: null,
        after: { txn_id: 'txn-1', tag_id: 'tag-1' },
      },
    ]);
    expect(refs).toEqual(
      expect.arrayContaining([
        { kind: 'txn', id: 'txn-1' },
        { kind: 'category', id: 'cat-outros' },
        { kind: 'category', id: 'cat-app' },
        { kind: 'tag', id: 'tag-1' },
      ]),
    );
    expect(refs).toHaveLength(4);
  });
});
