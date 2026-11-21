/**
 * Lançamento → parcela de dívida que ele pagou (espelho de `loan_installment.txn_id`). O
 * mock de dívidas responde (evita import circular): as visões de lançamento mostram a
 * parcela e apagar o lançamento reabre a parcela, como no núcleo.
 */
export interface LoanLink {
  loan_id: string;
  loan_name: string;
  installment_id: string;
  installment_number: number;
}

export interface LoanLookup {
  /** Parcela de cada lançamento ligado (a de menor número, se pagou várias). */
  links: () => ReadonlyMap<string, LoanLink>;
  /** Lançamentos apagados: as parcelas voltam a ficar em aberto. */
  forget: (txnIds: ReadonlySet<string>) => void;
}

let loans: LoanLookup = { links: () => new Map(), forget: () => undefined };

export function setLoanLookup(l: LoanLookup): void {
  loans = l;
}

export function loanLinks(): ReadonlyMap<string, LoanLink> {
  return loans.links();
}

export function forgetLoanTxns(ids: ReadonlySet<string>): void {
  loans.forget(ids);
}
