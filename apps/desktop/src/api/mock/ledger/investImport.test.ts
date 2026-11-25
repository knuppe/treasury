import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { B3CommitResult } from '@/api/generated/B3CommitResult';
import type { B3Preview } from '@/api/generated/B3Preview';
import type { NoteCommitResult } from '@/api/generated/NoteCommitResult';
import type { NotePreview } from '@/api/generated/NotePreview';
import type { Trade } from '@/api/generated/Trade';
import type { UncoveredSales } from '@/api/generated/UncoveredSales';
import { mockCall, resetMock } from '@/api/mock';
import { fixToday } from '@/features/portfolio/testing';

import { SAMPLE_NOTE } from '../notes';

const b64 = (s: string) => btoa(String.fromCharCode(...new TextEncoder().encode(s)));
const file = (filename: string, content = 'x') => ({ filename, content_base64: b64(content) });

async function trades(account_id?: string): Promise<Trade[]> {
  return (await mockCall('portfolio.list_trades', account_id ? { account_id } : {})) as Trade[];
}

describe('investment import mocks', () => {
  beforeEach(() => {
    fixToday();
    resetMock('unlocked');
  });
  afterEach(() => vi.useRealTimers());

  it('b3: preview with computed statuses, commit and undo through the history', async () => {
    const files = [file('negociacao-2026.xlsx'), file('movimentacao-2026.xlsx')];
    const p = (await mockCall('b3.preview', { files })) as B3Preview;
    const status = Object.fromEntries(
      p.trades.map((t) => [`${t.ticker}@${t.date}`, t.status] as const),
    );
    expect(status).toMatchObject({
      'PETR4@2026-03-10': 'possible_duplicate',
      'PETR4@2026-09-03': 'new',
      // No MGLU3 in this account on the date: imported all the same, with a pending item.
      'MGLU3@2026-09-22': 'warning',
      'BBAS3@2026-09-24': 'no_account',
      'KLBN11@2026-09-15': 'new',
    });
    expect(p.institutions.find((i) => i.name.startsWith('XP'))?.account_id).toBe('acc-xp');
    expect(p.assets.find((a) => a.key === 'KLBN11')).toMatchObject({
      create: true,
      needs_review: true,
    });
    expect(p.actions[0]!.status).toBe('review');
    const before = (await trades()).length;
    const r = (await mockCall('b3.commit', {
      token: p.token,
      approve_actions: [{ index: p.actions[0]!.index, factor: '0.1' }],
    })) as B3CommitResult;
    expect(p.counts.warnings).toBe(1);
    expect(p.counts.errors).toBe(0);
    expect(p.trades.find((t) => t.ticker === 'MGLU3')?.warning).toMatch(/maior que a posição/);
    expect(r.trades).toBe(p.counts.trades);
    // The sale waits for its purchase; the person may accept it without a cost.
    expect(r.pending_sales).toBe(1);
    const pending = (await mockCall('portfolio.uncovered_sales', {})) as UncoveredSales;
    expect(pending).toMatchObject({ open: 1, accepted: 0 });
    expect(pending.rows[0]).toMatchObject({
      ticker: 'MGLU3',
      missing_quantity: '300',
      missing_value: 273000,
    });
    const accepted = (await mockCall('portfolio.accept_uncovered_sale', {
      trade_id: pending.rows[0]!.trade_id,
      accepted: true,
    })) as UncoveredSales;
    expect(accepted).toMatchObject({ open: 0, accepted: 1 });
    expect(r.actions).toBe(1);
    expect(r.assets_created).toBe(1);
    expect((await trades()).length).toBe(before + r.trades);

    // Reimporting marks the same operations as duplicates.
    const again = (await mockCall('b3.preview', { files })) as B3Preview;
    expect(again.trades.find((t) => t.date === '2026-09-03')?.status).toBe('duplicate');

    await mockCall('changes.undo', { id: r.change_set_id });
    expect((await trades()).length).toBe(before);
    expect(await mockCall('assets.find', { ticker: 'KLBN11' })).toBeNull();
    await mockCall('changes.redo', { id: r.change_set_id });
    expect((await trades()).length).toBe(before + r.trades);
  });

  it('notes: password, unidentified asset, fill in costs and undo', async () => {
    // The HGLG11 sale already came from the B3 report, without costs.
    await mockCall('portfolio.create_trade', {
      asset_id: 'ast-hglg11',
      account_id: 'acc-xp',
      date: '2026-10-13',
      side: 'sell',
      quantity: '10',
      price: '160',
      cash_movement: false,
    });
    const files = [file('nota-senha.pdf', SAMPLE_NOTE)];
    const locked = (await mockCall('notes.preview', {
      files,
      account_id: 'acc-xp',
    })) as NotePreview;
    expect(locked.errors[0]?.needs_password).toBe(true);
    const p = (await mockCall('notes.preview', {
      files,
      account_id: 'acc-xp',
      password: '123',
    })) as NotePreview;
    expect(p.errors).toEqual([]);
    expect(p.notes[0]).toMatchObject({ number: '123456', net: -132126, computed_net: -132126 });
    expect(p.trades.map((t) => t.status)).toEqual(['new', 'unresolved', 'fill_costs']);
    expect(p.trades.map((t) => [t.fees, t.taxes])).toEqual([
      [75, 0],
      [1, 0],
      [42, 8],
    ]);
    expect(p.unresolved_specs).toEqual(['ITAUSA PN N1']);
    const r = (await mockCall('notes.commit', {
      files,
      account_id: 'acc-xp',
      password: '123',
      assets: [{ spec: 'ITAUSA PN N1', asset_id: 'ast-itub4' }],
    })) as NoteCommitResult;
    expect(r).toMatchObject({ created: 2, updated: 1, skipped: 0 });
    const sell = (await trades('acc-xp')).find((t) => t.date === '2026-10-13' && t.side === 'sell');
    expect(sell).toMatchObject({ fees: 42, taxes: 8, note_number: '123456' });
    await mockCall('changes.undo', { id: r.change_set_id });
    const after = (await trades('acc-xp')).find(
      (t) => t.date === '2026-10-13' && t.side === 'sell',
    );
    expect(after).toMatchObject({ fees: 0, taxes: 0, note_number: null });
  });
});
