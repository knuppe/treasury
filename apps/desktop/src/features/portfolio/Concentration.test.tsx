import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import { mockCall, mockCallLog } from '@/api/mock';
import type { AssetLabel } from '@/api/generated/AssetLabel';
import type { ConcentrationReport } from '@/api/generated/ConcentrationReport';
import { formatMoney } from '@/lib/format';
import { renderApp } from '@/test/renderApp';

import { PORTFOLIO_UI_DEFAULTS, usePortfolioUi } from './state';
import { fixToday, quietCanvas } from './testing';

beforeAll(quietCanvas);
beforeEach(() => {
  fixToday();
  usePortfolioUi.setState({ ...PORTFOLIO_UI_DEFAULTS });
});
afterEach(() => {
  vi.useRealTimers();
});

/** Opens the positions and applies `prepare` to the mock (after the `renderApp` reset). */
async function openPositions(prepare?: () => Promise<unknown>) {
  const app = renderApp('unlocked', {}, '#/carteira');
  if (prepare) {
    await prepare();
    await app.queryClient.invalidateQueries();
  }
  return app;
}

const buy = (asset_id: string, date: string) =>
  mockCall('portfolio.create_trade', {
    asset_id,
    account_id: 'acc-xp',
    date,
    side: 'buy',
    quantity: '10',
    price: '50',
  });

describe('08 · Carteira: recent position', () => {
  it('less than 30 days: "—" with the recent-position label and the period return', async () => {
    await openPositions(() => buy('ast-vale3', '2026-10-09'));
    await waitFor(() => {
      const r = screen.getAllByRole('row').find((x) => within(x).queryByText('VALE3'));
      expect(r && within(r).queryByText('posição recente')).toBeTruthy();
    });
    const r = screen.getAllByRole('row').find((x) => within(x).queryByText('VALE3'))!;
    expect(within(r).getByText('—', { selector: 'span.num' })).toBeTruthy();
    expect(within(r).getByText('no período')).toBeTruthy();
    // The hint says how many days ago and when the annualized one comes.
    expect(
      within(r).getByText(/Comprada há 5 dias\. Retorno no período: .+%\. O anualizado aparece/),
    ).toBeTruthy();
  });

  it('between 30 days and a year: the period return and the annualized one in discreet text', async () => {
    await openPositions(() => buy('ast-taee11', '2026-06-16'));
    await waitFor(() => {
      const r = screen.getAllByRole('row').find((x) => within(x).queryByText('TAEE11'));
      expect(r && within(r).queryByText(/anualizado sobre 120 dias/)).toBeTruthy();
    });
    const r = screen.getAllByRole('row').find((x) => within(x).queryByText('TAEE11'))!;
    expect(within(r).queryByText('posição recente')).toBeNull();
  });
});

describe('08 · Carteira: concentration', () => {
  it('the card shows the critical alert with the amount above the limit', async () => {
    let report: ConcentrationReport | undefined;
    await openPositions(async () => {
      report = (await mockCall('portfolio.set_concentration_limit', {
        scope: 'asset',
        asset_id: 'ast-aapl',
        pct: 5,
      })) as ConcentrationReport;
    });
    const alert = report!.alerts.find((a) => a.ref === 'ast-aapl')!;
    expect(alert.severity).toBe('critical');
    const list = await screen.findByRole('list', { name: 'Alertas de concentração' });
    const item = within(list)
      .getAllByRole('listitem')
      .find((li) => within(li).queryByText('AAPL'))!;
    expect(within(item).getByText('Bem acima do limite')).toBeTruthy();
    expect(within(item).getByText(/da carteira · limite 5,0%/)).toBeTruthy();
    expect(item.textContent).toContain(`${formatMoney(alert.excess_base)} acima do limite`);
    // The sample group (Exterior) also exceeds the 30% default.
    expect(within(list).getByText('Exterior')).toBeTruthy();
  });

  it('an asset limit in the dialog saves with portfolio.set_concentration_limit', async () => {
    const user = userEvent.setup();
    await openPositions();
    await user.click(await screen.findByRole('button', { name: 'Limites de concentração' }));
    const dialog = await screen.findByRole('dialog', { name: 'Limites de concentração' });
    const input = await within(dialog).findByLabelText('Limite de MSFT (%)');
    await user.type(input, '8');
    await user.click(within(dialog).getByRole('button', { name: 'Salvar limites' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(mockCallLog()).toContain('portfolio.set_concentration_limit');
    const report = (await mockCall('portfolio.concentration', {})) as ConcentrationReport;
    const msft = report.assets.find((a) => a.ref === 'ast-msft')!;
    expect(msft.custom_limit).toBe(true);
    expect(msft.limit).toBeCloseTo(0.08);
  });

  it('marking a label as a group saves with assets.label_update', async () => {
    const user = userEvent.setup();
    await openPositions();
    await user.click(await screen.findByRole('button', { name: 'Limites de concentração' }));
    const dialog = await screen.findByRole('dialog', { name: 'Limites de concentração' });
    await user.click(
      await within(dialog).findByRole('checkbox', { name: 'Bancos é grupo de concentração' }),
    );
    await user.type(within(dialog).getByLabelText('Limite do grupo Bancos (%)'), '5');
    await user.click(within(dialog).getByRole('button', { name: 'Salvar limites' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(mockCallLog()).toContain('assets.label_update');
    expect(mockCallLog()).not.toContain('assets.labels_set');
    const labels = (await mockCall('assets.labels_list', {})) as AssetLabel[];
    const bancos = labels.find((l) => l.name === 'Bancos')!;
    expect(bancos.is_concentration_group).toBe(true);
    expect(bancos.concentration_limit_pct).toBe(5);
    // ITUB4 + BBAS3 exceed 5%: the group enters the card.
    const list = await screen.findByRole('list', { name: 'Alertas de concentração' });
    expect(await within(list).findByText('Bancos')).toBeTruthy();
  });

  it('an asset labels save with assets.labels_set', async () => {
    const user = userEvent.setup();
    await openPositions();
    await user.click(await screen.findByRole('button', { name: 'Limites de concentração' }));
    const dialog = await screen.findByRole('dialog', { name: 'Limites de concentração' });
    await user.type(await within(dialog).findByLabelText('Etiquetas de PETR4'), 'Energia{Enter}');
    await user.click(within(dialog).getByRole('button', { name: 'Salvar limites' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    const labels = (await mockCall('assets.labels_list', {})) as AssetLabel[];
    expect(labels.find((l) => l.name === 'Energia')?.asset_ids).toEqual(['ast-petr4']);
  });
});
