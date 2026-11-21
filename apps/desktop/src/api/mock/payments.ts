/**
 * Mock de `payments.*` (docs/ocr-other-documents.md): a leitura sai pelo nome do arquivo ("boleto",
 * "pix", "comprovante"), com dados inventados; gravar cria o lançamento no razão do mock.
 */
import type { PaymentReading } from '../generated/PaymentReading';
import { mockFail, onMockReset, registerMockHandlers, requireUnlocked } from '../mockRegistry';

const BARCODE = '34191162000001234561234567890123456789012345';
const LINE = '34191.23456 67890.123459 67890.123459 1 16200000012345';

const planned = new Set<string>();
onMockReset(() => planned.clear());

function base(): Omit<PaymentReading, 'kind' | 'description'> {
  return {
    matches: [],
    planned: [],
    sha256: 'ab'.repeat(32),
    ocr: { engine: 'paddle', rows: ['Texto lido (exemplo)'], confidence: 0.9 },
    warnings: [],
  };
}

async function call(cmd: string, args: unknown): Promise<unknown> {
  const { mockCall } = await import('../mock');
  return mockCall(cmd, args);
}

async function read(args: Record<string, unknown>): Promise<PaymentReading> {
  requireUnlocked();
  const name = (typeof args.filename === 'string' ? args.filename : '').toLowerCase();
  const codes = Array.isArray(args.codes) ? (args.codes as string[]) : [];
  if (name.includes('comprovante')) {
    const page = (await call('transactions.query', { limit: 50 })) as {
      items?: {
        id: string;
        date: string;
        amount: number;
        description: string;
        account_id: string;
        account_name?: string;
        status: string;
      }[];
    };
    const m = (page.items ?? []).find((r) => r.amount < 0 && r.status !== 'planned');
    return {
      ...base(),
      kind: 'proof',
      description: 'Fulano de Teste',
      proof: {
        kind: 'pix',
        direction: 'out',
        amount: m ? -m.amount : 5_000,
        date: m?.date ?? '2026-10-22',
        time: '14:05',
        counterpart: 'Fulano de Teste',
        institution: 'Banco Exemplo',
        e2e_id: 'E18236120202610221705aBcDeFgHiJk',
        origin_bank: 'Nubank',
        scheduled: false,
        checked: true,
      },
      matches: m
        ? [
            {
              txn_id: m.id,
              date: m.date,
              amount: m.amount,
              description: m.description,
              account_id: m.account_id,
              account_name: m.account_name ?? 'Conta',
              status: m.status,
              attachments: 0,
            },
          ]
        : [],
    };
  }
  if (name.includes('pix')) {
    return {
      ...base(),
      kind: 'pix',
      description: 'Condomínio Exemplo',
      pix: {
        payload: '00020126580014br.gov.bcb.pix…6304ABCD',
        key: 'cobranca@exemplo.invalid',
        amount: 47_805,
        name: 'Condomínio Exemplo',
        city: 'Cidade',
        txid: 'TX1',
        dynamic: false,
      },
    };
  }
  if (name.includes('boleto') || codes.length) {
    return {
      ...base(),
      kind: 'bill',
      description: 'Escola Exemplo',
      bill: {
        kind: 'bank',
        barcode: BARCODE,
        line: LINE,
        amount: 12_345,
        due_date: '2026-11-10',
        bank_code: '341',
        bank_name: 'Itaú',
        beneficiary: 'Escola Exemplo',
      },
      ...(planned.has(BARCODE)
        ? {
            existing: {
              txn_id: 'mock-planned',
              date: '2026-11-10',
              amount: -12_345,
              description: 'Escola Exemplo',
              account_id: '',
              account_name: '',
              status: 'planned',
              attachments: 0,
            },
          }
        : {}),
    };
  }
  return { ...base(), kind: 'none', description: '' };
}

registerMockHandlers({
  'payments.read': read,
  'payments.plan': async (a) => {
    requireUnlocked();
    const code = typeof a.code === 'string' ? a.code : '';
    if (planned.has(code))
      mockFail('conflict', 'Este boleto ou Pix já está lançado como previsto.');
    planned.add(code);
    const t = (await call('transactions.create', {
      account_id: a.account_id,
      date: a.due_date,
      amount: -Number(a.amount),
      status: 'planned',
      description: a.description,
      category_id: a.category_id,
      notes: `Linha digitável: ${LINE}`,
    })) as { id: string };
    return t.id;
  },
  'payments.save_proof': async (a) => {
    requireUnlocked();
    const create = a.create as
      { account_id: string; amount: number; description: string; category_id?: string } | undefined;
    if (typeof a.txn_id === 'string') {
      await call('transactions.update', {
        id: a.txn_id,
        ...(typeof a.payee === 'string' ? { payee: a.payee } : {}),
      });
      return a.txn_id;
    }
    if (!create) mockFail('invalid', 'Escolha o lançamento do comprovante ou crie um.');
    const t = (await call('transactions.create', {
      account_id: create.account_id,
      date: a.date,
      amount: create.amount,
      description: create.description,
      payee: a.payee,
    })) as { id: string };
    return t.id;
  },
});
