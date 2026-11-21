import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import type { AccountView } from '@/api/generated/AccountView';

import { FreshnessBadge, freshnessHint } from './Freshness';

function account(patch: Partial<AccountView>): AccountView {
  return {
    id: 'a1',
    name: 'Corrente',
    kind: 'checking',
    currency: 'BRL',
    institution_id: null,
    institution_name: null,
    member_ids: [],
    opening_balance: 0,
    opening_date: '2026-01-01',
    include_in_net_worth: true,
    archived: false,
    color: null,
    notes: null,
    sort_order: 0,
    card: null,
    overdraft: null,
    branch: null,
    account_number: null,
    retirement_plan: null,
    retirement_coverage: null,
    balance: 10_000,
    projected_balance: 10_000,
    reconciled_balance: 0,
    balance_base: 10_000,
    fx_estimated: false,
    last_txn_date: '2026-10-10',
    last_import_at: null,
    staleness_days: 3,
    balance_status: 'fresh',
    missed_month: null,
    available_credit: null,
    overdraft_status: null,
    ...patch,
  };
}

describe('<FreshnessBadge>', () => {
  it('an up-to-date account shows nothing', () => {
    const { container } = render(<FreshnessBadge account={account({})} />);
    expect(container.textContent).toBe('');
  });

  it('when getting old, shows how many days ago it was updated', () => {
    render(<FreshnessBadge account={account({ balance_status: 'aging', staleness_days: 12 })} />);
    expect(screen.getByText('atualizado há 12 dias')).toBeTruthy();
  });

  it('possibly incomplete warns, and the hint states the last transaction and the negative one', () => {
    const a = account({
      balance_status: 'possibly_incomplete',
      staleness_days: 25,
      balance: -5_000,
      missed_month: '2026-09',
    });
    render(<FreshnessBadge account={a} />);
    expect(screen.getByText('possivelmente incompleto')).toBeTruthy();
    const hint = freshnessHint(a);
    expect(hint).toContain('10/10/2026');
    expect(hint).toContain('setembro de 2026');
    expect(hint).toContain('não necessariamente cheque especial');
  });
});
