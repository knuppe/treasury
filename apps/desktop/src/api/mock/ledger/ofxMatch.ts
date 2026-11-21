/**
 * Conta de um OFX no mock (espelho de `accounts::match_ofx`): primeiro a escolhida à mão na
 * última importação do mesmo arquivo (`accounts.ofx_refs`), depois banco, agência e número;
 * no cartão, o final.
 */
import type { OfxMatch } from '@/api/generated/OfxMatch';

import { onMockReset } from '@/api/mockRegistry';

import { bankByOfxId } from './bankDetails';
import { accountViews } from './domain';
import { find } from './store';

/** Lembrança `banco|agência|ACCTID|cartao` (ou `…|conta`) → id da conta. */
const refs = new Map<string, string>();
onMockReset(() => refs.clear());

const digits = (s: string | null | undefined) => (s ?? '').replace(/\D/g, '').replace(/^0+/, '');

function sameNumber(ofx: string, ours: string): boolean {
  const a = digits(ofx);
  const b = digits(ours);
  if (a.length < 3 || b.length < 3) return false;
  return (
    a === b ||
    a.endsWith(b) ||
    b.endsWith(a) ||
    a.endsWith(b.slice(0, -1)) ||
    b.endsWith(a.slice(0, -1))
  );
}

interface OfxRef {
  bank_id?: string | null;
  branch_id?: string | null;
  account_id?: string | null;
  credit_card?: boolean | null;
}

const opt = (v: unknown) => (typeof v === 'string' ? v : null);

function refOf(args: Record<string, unknown>): OfxRef {
  return {
    bank_id: opt(args.bank_id),
    branch_id: opt(args.branch_id),
    account_id: opt(args.account_id),
    credit_card: args.credit_card === true,
  };
}

/** Mesma chave do núcleo: banco e agência sem zeros à esquerda, `ACCTID` aparado. */
function refKey(r: OfxRef): string | null {
  const acct = (r.account_id ?? '').trim();
  if (!acct) return null;
  const part = (v: string | null | undefined) => (v ?? '').trim().replace(/^0+/, '').toLowerCase();
  const kind = r.credit_card ? 'cartao' : 'conta';
  return `${part(r.bank_id)}|${part(r.branch_id)}|${acct.toLowerCase()}|${kind}`;
}

/** Contas que combinam pelo cadastro (banco, agência e número; no cartão, o final). */
function autoMatch(r: OfxRef): string[] {
  const acct = (r.account_id ?? '').trim();
  if (!acct) return [];
  const card = r.credit_card === true;
  const bank = bankByOfxId(r.bank_id ?? undefined);
  const ofxBank = bank?.code ? digits(bank.code) : digits(r.bank_id);
  const ofxBranch = digits(r.branch_id);
  return accountViews(false)
    .filter((v) => (v.kind === 'credit_card') === card)
    .filter((v) => {
      if (card) {
        const last4 = v.card?.last4 ?? '';
        return last4 !== '' && acct.replace(/\D/g, '').endsWith(last4);
      }
      if (!v.account_number || !sameNumber(acct, v.account_number)) return false;
      const code = digits(v.institution_id ? find('institution', v.institution_id)?.code : null);
      if (code && ofxBank && code !== ofxBank) return false;
      const ours = digits(v.branch);
      if (ours && ofxBranch && ours !== ofxBranch) return false;
      return true;
    })
    .map((v) => v.id);
}

/** Conta lembrada, se ainda existe, não está arquivada e é do tipo do extrato. */
function remembered(r: OfxRef): string | null {
  const key = refKey(r);
  const id = key ? refs.get(key) : undefined;
  const acc = id ? find('account', id) : undefined;
  if (!acc || acc.archived || (acc.kind === 'credit_card') !== Boolean(r.credit_card)) return null;
  return acc.id;
}

export function matchOfx(args: Record<string, unknown>): OfxMatch {
  const r = refOf(args);
  const bank = bankByOfxId(r.bank_id ?? undefined);
  if (!(r.account_id ?? '').trim())
    return {
      account_id: null,
      bank,
      reason: 'o OFX não traz o número da conta',
      remembered: false,
    };
  const kept = remembered(r);
  if (kept) return { account_id: kept, bank, reason: null, remembered: true };
  const hits = autoMatch(r);
  const [only] = hits;
  if (hits.length === 1 && only) return { account_id: only, bank, reason: null, remembered: false };
  return {
    account_id: null,
    bank,
    reason:
      hits.length > 1
        ? 'mais de uma conta combina; escolha qual'
        : r.credit_card
          ? 'nenhum cartão com esse final; preencha o final do cartão no cadastro'
          : 'nenhuma conta com esse número; preencha banco, agência e conta no cadastro',
    remembered: false,
  };
}

/**
 * Espelho de `accounts::remember_ofx`: guarda a conta em que o OFX foi gravado quando o
 * cadastro não a reconhece sozinho; quando reconhece a mesma, a lembrança sai.
 */
export function rememberOfx(r: OfxRef, accountId: string) {
  const key = refKey(r);
  const acc = find('account', accountId);
  if (!key || !acc || (acc.kind === 'credit_card') !== Boolean(r.credit_card)) return;
  const auto = autoMatch(r);
  if (auto.length === 1 && auto[0] === accountId) refs.delete(key);
  else refs.set(key, accountId);
}
