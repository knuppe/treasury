/**
 * Agência e número da conta no mock (espelho de `accounts::bank_details`) e uma amostra da
 * lista de bancos do Banco Central (`household.banks`).
 */
import type { Bank } from '@/api/generated/Bank';

import { onMockReset } from '@/api/mockRegistry';

import { invalid } from './errors';

const details = new Map<string, { branch: string | null; account_number: string | null }>();
onMockReset(() => details.clear());

export function bankDetailsOf(id: string): {
  branch: string | null;
  account_number: string | null;
} {
  return details.get(id) ?? { branch: null, account_number: null };
}

/** Mesma regra do núcleo: dígitos, hífen e "X", sem espaços nem pontos. */
export function cleanBankRef(v: unknown, what: string): string | null {
  if (typeof v !== 'string') return null;
  const s = v.replace(/[\s.]/g, '');
  if (!s) return null;
  if (s.length > 30 || !/^[0-9xX-]+$/.test(s)) invalid(`${what}: use só números e hífen`);
  return s.toUpperCase();
}

export function setBankDetails(id: string, branch: string | null, accountNumber: string | null) {
  if (branch == null && accountNumber == null) details.delete(id);
  else details.set(id, { branch, account_number: accountNumber });
}

export const BANKS: Bank[] = [
  {
    ispb: '00000000',
    code: '001',
    short_name: 'BCO DO BRASIL S.A.',
    full_name: 'Banco do Brasil S.A.',
  },
  {
    ispb: '00360305',
    code: '104',
    short_name: 'CAIXA ECONOMICA FEDERAL',
    full_name: 'Caixa Econômica Federal',
  },
  {
    ispb: '60746948',
    code: '237',
    short_name: 'BCO BRADESCO S.A.',
    full_name: 'Banco Bradesco S.A.',
  },
  {
    ispb: '60701190',
    code: '341',
    short_name: 'ITAÚ UNIBANCO S.A.',
    full_name: 'Itaú Unibanco S.A.',
  },
  {
    ispb: '90400888',
    code: '033',
    short_name: 'BCO SANTANDER (BRASIL) S.A.',
    full_name: 'Banco Santander (Brasil) S.A.',
  },
  {
    ispb: '18236120',
    code: '260',
    short_name: 'NU PAGAMENTOS - IP',
    full_name: 'Nu Pagamentos S.A. - Instituição de Pagamento',
  },
  { ispb: '00416968', code: '077', short_name: 'BANCO INTER', full_name: 'Banco Inter S.A.' },
  {
    ispb: '05463212',
    code: '756',
    short_name: 'BANCO SICOOB S.A.',
    full_name: 'Banco Cooperativo Sicoob S.A.',
  },
  {
    ispb: '01181521',
    code: '748',
    short_name: 'BCO COOPERATIVO SICREDI S.A.',
    full_name: 'Banco Cooperativo Sicredi S.A.',
  },
  { ispb: '31872495', code: '336', short_name: 'BCO C6 S.A.', full_name: 'Banco C6 S.A.' },
  {
    ispb: '30306294',
    code: '208',
    short_name: 'BANCO BTG PACTUAL S.A.',
    full_name: 'Banco BTG Pactual S.A.',
  },
  {
    ispb: '10573521',
    code: '323',
    short_name: 'MERCADO PAGO IP LTDA.',
    full_name: 'Mercado Pago Instituição de Pagamento Ltda.',
  },
];

const fold = (s: string) => s.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase();

export function searchBanks(query: string | undefined, limit = 20): Bank[] {
  const q = fold((query ?? '').trim());
  const hits = q
    ? BANKS.filter(
        (b) =>
          fold(b.short_name).includes(q) ||
          fold(b.full_name).includes(q) ||
          b.ispb.startsWith(q) ||
          (b.code != null && (b.code === q.padStart(3, '0') || b.code.startsWith(q))),
      )
    : BANKS;
  return hits.slice(0, Math.max(1, Math.min(limit, 100)));
}

export function bankByOfxId(id: string | undefined): Bank | null {
  const d = (id ?? '').replace(/\D/g, '');
  if (!d) return null;
  const n = d.replace(/^0+/, '');
  return (
    BANKS.find((b) => b.code != null && b.code.replace(/^0+/, '') === n) ??
    BANKS.find((b) => b.ispb === d.padStart(8, '0')) ??
    null
  );
}
